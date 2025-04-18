use std::char;

use crate::error::{PdfError, Result};
use crate::font::cmap::{Cmap, CodeSpaceRange};
use crate::object::{number::PdfNumber, PdfObject};
use crate::reader::{PdfReader, Token};

fn decode_utf16be(bytes: &[u8]) -> String {
    let (cs, _enc, has_err) = encoding_rs::UTF_16BE.decode(bytes);
    return cs.to_string();
}

pub struct CmapParser {
    reader: PdfReader,
}

fn u32_from_hex(buf: &[u8]) -> Result<u32> {
    let mut s = String::new();
    for c in buf {
        s.push(c.to_owned() as char);
    }

    let v = u32::from_str_radix(s.as_str(), 16)
        .map_err(|e| PdfError::Font(format!("Cmap u32 from hex error:{:?}", e)))?;
    Ok(v)
}

impl CmapParser {
    pub fn new(data: Vec<u8>) -> Self {
        let s = String::from_utf8(data.clone()).unwrap();
        let reader = PdfReader::new(data);
        Self { reader }
    }

    fn process_code_space_range(&self, cmap: &mut Cmap, n: usize) -> Result<()> {
        for _ in 0..n {
            let low = self.reader.read_object()?.as_hex_string()?.to_owned();
            let high = self.reader.read_object()?.as_hex_string()?.to_owned();
            let l = u32_from_hex(low.bytes())?;
            let h = u32_from_hex(high.bytes())?;
            let size = high.bytes().len() / 2;
            cmap.add_code_space_range(CodeSpaceRange::new(l, h, size as u8));
        }
        Ok(())
    }

    fn process_cid_range(&self, cmap: &mut Cmap, n: usize) -> Result<()> {
        for _ in 0..n {
            let start = self.reader.read_object()?.as_hex_string()?.to_owned();
            let end = self.reader.read_object()?.as_hex_string()?.to_owned();
            let val = self.reader.read_object()?.as_number()?.to_owned();
            let l = u32_from_hex(start.bytes())?;
            let h = u32_from_hex(end.bytes())?;
            let mut c = val.integer() as u32;
            for code in l..h {
                cmap.add_code_to_cid(code, c);
                c += 1;
            }
        }
        Ok(())
    }

    fn process_cid_char(&self, cmap: &mut Cmap, n: usize) -> Result<()> {
        for _ in 0..n {
            let key = self.reader.read_object()?.as_hex_string()?.to_owned();
            let val = self.reader.read_object()?;
            let cid = u32_from_hex(key.bytes())?;
            match val {
                PdfObject::HexString(s) => {
                    cmap.add_code_to_cid(cid, u32_from_hex(s.bytes())?);
                }
                PdfObject::Number(n) => {
                    cmap.add_code_to_cid(cid, n.integer() as u32);
                }
                _ => {
                    return Err(PdfError::Font(format!(
                        "Cmap process cid range get{:?} ",
                        val
                    )));
                }
            }
        }
        Ok(())
    }

    fn process_bf_range(&self, cmap: &mut Cmap, n: usize) -> Result<()> {
        for _ in 0..n {
            let _ = self.reader.read_token()?;
            let low = self.reader.read_hex_string()?;
            let high = self.reader.read_object()?.as_hex_string()?.to_owned();
            let l = u32_from_hex(low.bytes())?;
            let h = u32_from_hex(high.bytes())?;
            let obj = self.reader.read_object()?;
            match obj {
                PdfObject::Array(arr) => {
                    for code in l..=h {
                        let index = (code - l) as usize;
                        let v = arr.get(index).unwrap().as_hex_string()?.to_owned();
                        let unicode = decode_utf16be(v.raw_bytes()?.as_slice());
                        cmap.add_unicode(code, unicode);
                    }
                }
                PdfObject::HexString(s) => {
                    let mut unicode = u32_from_hex(s.bytes())?;
                    for code in l..=h {
                        let c = char::from_u32(unicode).unwrap_or(char::REPLACEMENT_CHARACTER);
                        let mut s = String::new();
                        s.push(c);
                        cmap.add_unicode(code, s);
                        unicode += 1;
                    }
                }
                _ => {
                    return Err(PdfError::Font(format!(
                        "Cmap process bfrange value need Hexstring or PdfArray got:{:?}",
                        obj
                    )));
                }
            }
        }
        Ok(())
    }

    fn process_bf_char(&self, cmap: &mut Cmap, n: usize) -> Result<()> {
        for _ in 0..n {
            let t = self.reader.read_token()?;
            if t.is_other_key(b"endbfchar") {
                break;
            }
            let code = self.reader.read_hex_string()?;
            let _ = self.reader.read_token()?;
            let c = self.reader.read_hex_string()?;
            let kbyte = code.bytes();
            let unicode = decode_utf16be(c.raw_bytes()?.as_slice());
            cmap.add_unicode(u32_from_hex(kbyte)?, unicode);
        }
        Ok(())
    }

    pub fn parse(&self) -> Result<Cmap> {
        let mut args: Vec<PdfObject> = Vec::new();
        let mut cmap = Cmap::default();
        while !self.reader.is_eof() {
            let token = self.reader.read_token()?;
            if token.is_other_key(b"endcmap") {
                break;
            }
            match token {
                Token::Other(buf) => match buf {
                    b"usecmap" => {
                        let name = args
                            .pop()
                            .ok_or(PdfError::Font("Cmap usecmap key is None".to_string()))?
                            .as_name()
                            .map_err(|_| PdfError::Font("usecmap arg is not a name".to_string()))?
                            .to_owned();
                        let ocmp = Cmap::new_from_predefined(name.name())?;
                        cmap.usecmap(ocmp);
                    }
                    b"begincodespacerange" => {
                        let n = args.pop().unwrap().integer()?;
                        self.process_code_space_range(&mut cmap, n as usize)?;
                    }
                    b"beginbfchar" => {
                        let n = args.pop().unwrap().integer()?;
                        self.process_bf_char(&mut cmap, n as usize)?;
                    }
                    b"beginbfrange" => {
                        let n = args.pop().unwrap().integer()?;
                        self.process_bf_range(&mut cmap, n as usize)?;
                    }
                    b"def" => {
                        let val = args.pop();
                        let key = args.pop().unwrap();
                        match key {
                            PdfObject::Name(name) => {
                                if name.name() == "CMapName" {
                                    cmap.name = val.unwrap().as_name()?.name().to_string();
                                } else if name.name() == "WMode" {
                                    cmap.wmode = Some(val.unwrap().as_number()?.integer() as u8);
                                }
                            }
                            _ => {}
                        }
                    }
                    b"begincidchar" => {
                        let n = args.pop().unwrap().integer()?;
                        self.process_cid_char(&mut cmap, n as usize)?;
                    }
                    b"begincidrange" => {
                        let n = args.pop().unwrap().integer()?;
                        self.process_cid_range(&mut cmap, n as usize)?;
                    }
                    _ => {}
                },
                Token::StartComment => {
                    self.reader.read_comment()?;
                }
                Token::StartHexString => {
                    let s = self.reader.read_hex_string()?;
                    args.push(PdfObject::HexString(s));
                }
                Token::StartDict => {
                    let d = self.reader.read_dict()?;
                    args.push(PdfObject::Dict(d));
                }
                Token::StartLiteralString => {
                    let s = self.reader.read_literial_string()?;
                    args.push(PdfObject::LiteralString(s));
                }
                Token::Number(buf, is_real) => {
                    let num = PdfNumber::from_buffer(buf, is_real);
                    args.push(PdfObject::Number(num));
                }
                Token::StartName => {
                    let name = self.reader.read_name()?;
                    args.push(PdfObject::Name(name));
                }
                Token::StartArray => {
                    let a = self.reader.read_array()?;
                    args.push(PdfObject::Array(a));
                }
                _ => {
                    println!("impossiable token:{:?}", token)
                }
            }
        }
        Ok(cmap)
    }
}

#[cfg(test)]
mod tests {
    use crate::font::cmap::parser::CmapParser;

    #[test]
    fn test_cmap_parse() {
        let content = b"
%! commant
%% commant
/CIDInit /ProcSet findresource begin 12 dict begin begincmap /CIDSystemInfo <<
/Registry (AAAAAA+F4+0) /Ordering (T1UV) /Supplement 0 >> def
/CMapName /AAAAAA+F4+0 def
/CMapType 2 def
1 begincodespacerange <18> <fc> endcodespacerange
15 beginbfchar
<18> <02D8>
<19> <02C7>
<21> <0021>
<5d> <005D>
<5f> <005F>
<84> <2014>
<85> <2013>
<b8> <00B8>
<e4> <00E4>
<e9> <00E9>
<ed> <00ED>
<ef> <00EF>
<f4> <00F4>
<f6> <00F6>
<fc> <00FC>
endbfchar
9 beginbfrange
<23> <26> <0023>
<28> <3b> <0028>
<3d> <3f> <003D>
<41> <5b> <0041>
<61> <7e> <0061>
<8d> <8e> <201C>
<8f> <90> <2018>
<93> <94> <FB01>
<e0> <e1> <00E0>
endbfrange
endcmap CMapName currentdict /CMap defineresource pop end end";
        let cmap_parser = CmapParser::new(content.to_vec());
        let cmap = cmap_parser.parse().unwrap();
        for (key, v) in cmap.cid_to_unicode.iter() {
            println!("{:?},{:?}", key, v);
        }
        //println!("{:?}", cmap);
    }

    #[test]
    fn test_parse_cid() {
        let content = b"%!PS-Adobe-3.0 Resource-CMap
%%DocumentNeededResources: ProcSet (CIDInit)
%%IncludeResource: ProcSet (CIDInit)
%%BeginResource: CMap (83pv-RKSJ-H)
%%Title: (83pv-RKSJ-H Adobe Japan1 1)
%%Version: 10.005
%%Copyright: -----------------------------------------------------------
%%Copyright: Copyright 1990-2015 Adobe Systems Incorporated.
%%Copyright: All rights reserved.
%%Copyright:
%%Copyright: Redistribution and use in source and binary forms, with or
%%Copyright: without modification, are permitted provided that the
%%Copyright: following conditions are met:
%%Copyright:
%%Copyright: Redistributions of source code must retain the above
%%Copyright: copyright notice, this list of conditions and the following
%%Copyright: disclaimer.
%%Copyright:
%%Copyright: Redistributions in binary form must reproduce the above
%%Copyright: copyright notice, this list of conditions and the following
%%Copyright: disclaimer in the documentation and/or other materials
%%Copyright: provided with the distribution.
%%Copyright:
%%Copyright: Neither the name of Adobe Systems Incorporated nor the names
%%Copyright: of its contributors may be used to endorse or promote
%%Copyright: products derived from this software without specific prior
%%Copyright: written permission.
%%Copyright:
%%Copyright: THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND
%%Copyright: INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
%%Copyright: MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
%%Copyright: DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
%%Copyright: CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
%%Copyright: SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
%%Copyright: NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
%%Copyright: LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
%%Copyright: HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
%%Copyright: CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR
%%Copyright: OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
%%Copyright: SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
%%Copyright: -----------------------------------------------------------
%%EndComments

/CIDInit /ProcSet findresource begin

12 dict begin

begincmap

/CIDSystemInfo 3 dict dup begin
  /Registry (Adobe) def
  /Ordering (Japan1) def
  /Supplement 1 def
end def

/CMapName /83pv-RKSJ-H def
/CMapVersion 10.005 def
/CMapType 1 def

/UIDOffset 0 def
/XUID [1 10 25324] def

/WMode 0 def

5 begincodespacerange
  <00>   <80>
  <8140> <9FFC>
  <A0>   <DF>
  <E040> <FCFC>
  <FD>   <FF>
endcodespacerange

1 beginnotdefrange
<00> <1f> 1
endnotdefrange

100 begincidrange
<20> <7e>        1
<80> <80>       97
<8140> <817e>  633
<8180> <81ac>  696
<81b8> <81bf>  741
<81c8> <81ce>  749
<81da> <81e8>  756
<81f0> <81f7>  771
<81fc> <81fc>  779
<824f> <8258>  780
<8260> <8279>  790
<8281> <829a>  816
<829f> <82f1>  842
<8340> <837e>  925
<8380> <8396>  988
<839f> <83b6> 1011
<83bf> <83d6> 1035
<8440> <8460> 1059
<8470> <847e> 1092
<8480> <8491> 1107
<849f> <849f> 7479
<84a0> <84a0> 7481
<84a1> <84a1> 7491
<84a2> <84a2> 7495
<84a3> <84a3> 7503
<84a4> <84a4> 7499
<84a5> <84a5> 7507
<84a6> <84a6> 7523
<84a7> <84a7> 7515
<84a8> <84a8> 7531
<84a9> <84a9> 7539
<84aa> <84aa> 7480
<84ab> <84ab> 7482
<84ac> <84ac> 7494
<84ad> <84ad> 7498
<84ae> <84ae> 7506
<84af> <84af> 7502
<84b0> <84b0> 7514
<84b1> <84b1> 7530
<84b2> <84b2> 7522
<84b3> <84b3> 7538
<84b4> <84b4> 7554
<84b5> <84b5> 7511
<84b6> <84b6> 7526
<84b7> <84b7> 7519
<84b8> <84b8> 7534
<84b9> <84b9> 7542
<84ba> <84ba> 7508
<84bb> <84bb> 7527
<84bc> <84bc> 7516
<84bd> <84bd> 7535
<84be> <84be> 7545
<8540> <857e>  232
<8580> <8580>  390
<8581> <859e>  296
<859f> <85dd>  327
<85de> <85fc>  391
<8640> <867e>  422
<8680> <8691>  485
<8692> <8692>  295
<8693> <869e>  503
<86a2> <86ed> 7479
<8740> <875d> 7555
<875f> <8775> 7585
<8780> <878f> 7608
<8790> <8790>  762
<8791> <8791>  761
<8792> <8792>  769
<8793> <8799> 7624
<879a> <879a>  768
<879b> <879c> 7631
<889f> <88fc> 1125
<8940> <897e> 1219
<8980> <89fc> 1282
<8a40> <8a7e> 1407
<8a80> <8afc> 1470
<8b40> <8b7e> 1595
<8b80> <8bfc> 1658
<8c40> <8c7e> 1783
<8c80> <8cfc> 1846
<8d40> <8d7e> 1971
<8d80> <8dfc> 2034
<8e40> <8e7e> 2159
<8e80> <8efc> 2222
<8f40> <8f7e> 2347
<8f80> <8ffc> 2410
<9040> <907e> 2535
<9080> <90fc> 2598
<9140> <917e> 2723
<9180> <91fc> 2786
<9240> <927e> 2911
<9280> <92fc> 2974
<9340> <937e> 3099
<9380> <93fc> 3162
<9440> <947e> 3287
<9480> <94fc> 3350
<9540> <957e> 3475
<9580> <95fc> 3538
<9640> <967e> 3663
<9680> <96fc> 3726
endcidrange

100 begincidrange
<9740> <977e> 3851
<9780> <97fc> 3914
<9840> <9872> 4039
<989f> <98fc> 4090
<9940> <997e> 4184
<9980> <99fc> 4247
<9a40> <9a7e> 4372
<9a80> <9afc> 4435
<9b40> <9b7e> 4560
<9b80> <9bfc> 4623
<9c40> <9c7e> 4748
<9c80> <9cfc> 4811
<9d40> <9d7e> 4936
<9d80> <9dfc> 4999
<9e40> <9e7e> 5124
<9e80> <9efc> 5187
<9f40> <9f7e> 5312
<9f80> <9ffc> 5375
<a0> <df>      326
<e040> <e07e> 5500
<e080> <e0fc> 5563
<e140> <e17e> 5688
<e180> <e1fc> 5751
<e240> <e27e> 5876
<e280> <e2fc> 5939
<e340> <e37e> 6064
<e380> <e3fc> 6127
<e440> <e47e> 6252
<e480> <e4fc> 6315
<e540> <e57e> 6440
<e580> <e5fc> 6503
<e640> <e67e> 6628
<e680> <e6fc> 6691
<e740> <e77e> 6816
<e780> <e7fc> 6879
<e840> <e87e> 7004
<e880> <e8fc> 7067
<e940> <e97e> 7192
<e980> <e9fc> 7255
<ea40> <ea7e> 7380
<ea80> <eaa2> 7443
<eaa3> <eaa4> 8284
<eb40> <eb40>  633
<eb41> <eb42> 7887
<eb43> <eb4f>  636
<eb50> <eb51> 7889
<eb52> <eb5a>  651
<eb5b> <eb5d> 7891
<eb5e> <eb5f>  663
<eb60> <eb64> 7894
<eb65> <eb68>  670
<eb69> <eb7a> 7899
<eb7b> <eb7e>  692
<eb80> <eb80>  696
<eb81> <eb81> 7917
<eb82> <ebac>  698
<ebb8> <ebbf>  741
<ebc8> <ebce>  749
<ebda> <ebe8>  756
<ebf0> <ebf7>  771
<ebfc> <ebfc>  779
<ec4f> <ec58>  780
<ec60> <ec79>  790
<ec81> <ec9a>  816
<ec9f> <ec9f> 7918
<eca0> <eca0>  843
<eca1> <eca1> 7919
<eca2> <eca2>  845
<eca3> <eca3> 7920
<eca4> <eca4>  847
<eca5> <eca5> 7921
<eca6> <eca6>  849
<eca7> <eca7> 7922
<eca8> <ecc0>  851
<ecc1> <ecc1> 7923
<ecc2> <ece0>  877
<ece1> <ece1> 7924
<ece2> <ece2>  909
<ece3> <ece3> 7925
<ece4> <ece4>  911
<ece5> <ece5> 7926
<ece6> <eceb>  913
<ecec> <ecec> 7927
<eced> <ecf1>  920
<ed40> <ed40> 7928
<ed41> <ed41>  926
<ed42> <ed42> 7929
<ed43> <ed43>  928
<ed44> <ed44> 7930
<ed45> <ed45>  930
<ed46> <ed46> 7931
<ed47> <ed47>  932
<ed48> <ed48> 7932
<ed49> <ed61>  934
<ed62> <ed62> 7933
<ed63> <ed7e>  960
<ed80> <ed82>  988
<ed83> <ed83> 7934
<ed84> <ed84>  992
<ed85> <ed85> 7935
endcidrange

22 begincidrange
<ed86> <ed86>  994
<ed87> <ed87> 7936
<ed88> <ed8d>  996
<ed8e> <ed8e> 7937
<ed8f> <ed94> 1003
<ed95> <ed96> 7938
<ed9f> <edb6> 1011
<edbf> <edd6> 1035
<ee40> <ee5d> 7555
<ee5f> <ee6e> 7940
<ee6f> <ee75> 7601
<ee80> <ee81> 7956
<ee82> <ee8f> 7610
<ee90> <ee90>  762
<ee91> <ee91>  761
<ee92> <ee92>  769
<ee93> <ee99> 7624
<ee9a> <ee9a>  768
<ee9b> <ee9c> 7631
<fd> <fd>      152
<fe> <fe>      228
<ff> <ff>      124
endcidrange
endcmap
CMapName currentdict /CMap defineresource pop
end
end

%%EndResource
%%EOF";

        let cmap_parser = CmapParser::new(content.to_vec());
        let cmap = cmap_parser.parse().unwrap();
        assert_eq!(cmap.code_to_cids.len(), 7768_usize);
    }
}
