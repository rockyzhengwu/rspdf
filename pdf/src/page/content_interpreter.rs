use std::io::{Read, Seek};
use std::u8;

use log::warn;

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::geom::matrix::Matrix;
use crate::geom::path::Path;
use crate::geom::point::Point;
use crate::geom::rectangle::Rectangle;
use crate::object::{PDFNumber, PDFObject, PDFString};
use crate::page::content_parser::ContentParser;
use crate::page::graphics_object::GraphicsObject;
use crate::page::graphics_state::GraphicsState;
use crate::page::image::Image;
use crate::page::operation::Operation;
use crate::page::page_path::PagePath;
use crate::page::text::{Text, TextOpItem};
use crate::page::Page;

pub struct ContentInterpreter<'a, T: Seek + Read> {
    doc: &'a Document<T>,
    page: &'a Page<'a, T>,
    parser: ContentParser,
    state_stack: Vec<GraphicsState>,
    cur_state: GraphicsState,
    resource: Option<PDFObject>,
    current_path: Path,
}

impl<'a, T: Seek + Read> ContentInterpreter<'a, T> {
    pub fn try_new(page: &'a Page<'_, T>, doc: &'a Document<T>) -> PDFResult<Self> {
        let contents = page.contents()?;
        let mut buffers = Vec::new();
        for content in contents {
            buffers.extend(content.bytes());
        }
        let sss = String::from_utf8_lossy(buffers.as_slice());
        for item in sss.split('\n') {
            println!("{:?}", item);
        }
        let parser = ContentParser::try_new(buffers)?;
        Ok(ContentInterpreter {
            page,
            doc,
            parser,
            state_stack: Vec::new(),
            resource: None,
            cur_state: GraphicsState::default(),
            current_path: Path::default(),
        })
    }
    pub fn start(&mut self) -> PDFResult<()> {
        let resource = self.page.resources()?;
        let state = GraphicsState::default();
        self.resource = Some(resource);
        self.state_stack.push(state);
        Ok(())
    }

    pub fn poll(&mut self) -> PDFResult<Option<GraphicsObject>> {
        loop {
            let op = self.parser.parse_operation();
            if op.is_err() {
                break;
            }
            let res = self.invoke_operation(op.unwrap())?;
            if let Some(obj) = res {
                return Ok(Some(obj));
            }
        }
        Ok(None)
    }

    fn invoke_operation(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let op = operation.name();
        match op {
            // default
            "q" => self.push_graph_state(),
            "Q" => self.pop_graph_state(),
            "cm" => self.modify_current_transform_matrix(operation),
            "Do" => self.do_operation(operation),
            "BMC" => self.begin_marked_content(operation),
            "BDC" => self.begin_marked_content_dictionary(operation),
            "EMC" => self.end_marked_content(operation),
            // text
            "BT" => self.begin_text(),
            "ET" => self.end_text(),
            "Tc" => self.set_text_character_spacing(operation),
            "Tw" => self.set_text_word_spacing(operation),
            "Tz" => self.set_text_horizal_scaling(operation),
            "TL" => self.set_text_leading(operation),
            "Tf" => self.set_text_font(operation),
            "Tr" => self.set_text_reander_mode(operation),
            "Ts" => self.set_text_rise_mode(operation),
            "Td" => self.text_move_start_next_line(operation),
            "TD" => self.text_move_start_next_line_with_leading(operation),
            "Tm" => self.text_set_text_matrix(operation),
            "T*" => self.text_set_move_next_line(),
            "Tj" => self.show_text(operation),
            "'" => self.move_next_line_and_show_text(operation),
            "\"" => self.move_text_line_and_show_text_with_leading(operation),
            "TJ" => self.show_text_array(operation),
            // path
            "w" => self.set_line_width(operation),
            "J" => self.set_line_cap(operation),
            "j" => self.set_line_join(operation),
            "M" => self.set_miter_limit(operation),
            "d" => self.set_line_dash_pattern(operation),
            "m" => self.move_to(operation),
            "l" => self.line_to(operation),
            "c" => self.curve(operation),
            "v" => self.curve_first_point_duplicate(operation),
            "re" => self.reanctle(operation),
            "y" => self.curve_fourh_point_duplicate(operation),
            "h" => self.close_sub_path(operation),
            "S" | "s" | "F" | "f*" | "B" | "B*" | "b" | "b*" | "n" | "f" => {
                self.paint_path(operation)
            }
            //
            "g" => self.set_gray_fill(operation),
            "G" => self.set_gray_stroke(operation),
            "rg" => self.set_rgb_fill(operation),
            "RG" => self.set_rgb_stroke(operation),
            "k" => self.set_cmyk_fill(operation),
            "K" => self.set_cmyk_stroke(operation),
            "cs" => self.set_color_space_fill(operation),
            "CS" => self.set_color_space_stroke(operation),
            "sc" | "scn" => self.set_color_fill(operation),
            "SC" | "SCN" => self.set_color_stroke(operation),
            "gs" => self.set_extend_graphic_state(operation),
            // Image
            "EI" => self.end_image(operation),
            _ => Ok(None),
        }
    }

    fn find_resource(&self, key: &str, name: &str) -> PDFResult<Option<PDFObject>> {
        if let Some(resource) = self.resource.as_ref() {
            if let Some(entry) = resource.get_value(key) {
                let container = self.doc.get_object_without_indriect(entry)?;
                let result = container.get_value(name);
                if let Some(v) = result {
                    let vv = self.doc.get_object_without_indriect(v)?;
                    return Ok(Some(vv));
                }
            }
        }
        Ok(None)
    }

    // EI
    fn end_image(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        println!("inline image {:?}", operation);
        Ok(None)
    }

    // gs
    fn set_extend_graphic_state(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        let name = operation.operand(0)?.as_string()?;
        let _ext_state = self.find_resource("ExtGState", name.as_str())?;
        Ok(None)
    }

    // SC
    fn set_color_stroke(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }
    // sc
    fn set_color_fill(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }

    // CS
    fn set_color_space_stroke(
        &mut self,
        _operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }

    // cs
    fn set_color_space_fill(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let ope = operation.operand(0)?.as_string()?;
        let _color_space = self.find_resource("ColorSpace", &ope)?;
        println!("colorspace operation {:?}", _color_space);
        Ok(None)
    }
    // K
    fn set_cmyk_stroke(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }
    // k
    fn set_cmyk_fill(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }
    //RG
    fn set_rgb_stroke(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }
    // rg
    fn set_rgb_fill(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }
    // G
    fn set_gray_stroke(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        //pass
        Ok(None)
    }
    // g
    fn set_gray_fill(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }

    // S
    fn paint_path(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        // TODO implement paint path
        // let state = self.last_mut_state().clone();
        // self.current_path.close_last_subpath();
        let mut path = std::mem::take(&mut self.current_path);
        path.close_last_subpath();
        let page_path = PagePath::new(path, self.cur_state.clone());
        // self.device.borrow_mut().paint_path(&path)?;
        Ok(Some(GraphicsObject::Path(page_path)))
    }

    // re
    fn reanctle(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let lx = operation.operand(0)?.as_f64()?;
        let ly = operation.operand(1)?.as_f64()?;
        let ux = operation.operand(2)?.as_f64()?;
        let uy = operation.operand(3)?.as_f64()?;
        self.current_path.rectangle(Rectangle::new(lx, ly, ux, uy));
        Ok(None)
    }

    // h
    fn close_sub_path(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }

    // y
    fn curve_fourh_point_duplicate(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        let x1 = operation.operand(0)?.as_f64()?;
        let y1 = operation.operand(0)?.as_f64()?;
        let x3 = operation.operand(0)?.as_f64()?;
        let y3 = operation.operand(0)?.as_f64()?;

        let p1 = Point::new(x1, y1);
        let p3 = Point::new(x3, y3);
        self.current_path.curve_to(vec![p1, p3, p3]);
        Ok(None)
    }

    // v
    fn curve_first_point_duplicate(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        let x2 = operation.operand(0)?.as_f64()?;
        let y2 = operation.operand(0)?.as_f64()?;
        let x3 = operation.operand(0)?.as_f64()?;
        let y3 = operation.operand(0)?.as_f64()?;

        let p2 = Point::new(x2, y2);
        let p3 = Point::new(x3, y3);
        self.current_path.curve_to(vec![p2, p3]);
        Ok(None)
    }

    // c
    fn curve(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let x1 = operation.operand(0)?.as_f64()?;
        let y1 = operation.operand(0)?.as_f64()?;
        let x2 = operation.operand(0)?.as_f64()?;
        let y2 = operation.operand(0)?.as_f64()?;
        let x3 = operation.operand(0)?.as_f64()?;
        let y3 = operation.operand(0)?.as_f64()?;
        let p1 = Point::new(x1, y1);
        let p2 = Point::new(x2, y2);
        let p3 = Point::new(x3, y3);
        self.current_path.curve_to(vec![p1, p2, p3]);

        Ok(None)
    }

    // l lineto
    fn line_to(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let x = operation.operand(0)?.as_f64()?;
        let y = operation.operand(1)?.as_f64()?;
        let p = Point::new(x, y);
        self.current_path.line_to(p);
        Ok(None)
    }

    // m moveto
    fn move_to(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let x = operation.operand(0)?.as_f64()?;
        let y = operation.operand(1)?.as_f64()?;
        let p = Point::new(x, y);
        self.current_path.move_to(p);
        Ok(None)
    }

    // d
    fn set_line_dash_pattern(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let pattern = operation.operand(0)?.as_array()?;
        let mut dash = Vec::new();
        for v in pattern {
            dash.push(v.as_f64()?);
        }
        self.cur_state.path_state.set_dash_array(dash);
        Ok(None)
    }

    // M
    fn set_miter_limit(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let limit = operation.operand(0)?;
        self.cur_state.path_state.set_miter_limit(limit.as_f64()?);
        Ok(None)
    }

    // w
    fn set_line_width(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let w = operation.operand(0)?;
        self.cur_state.path_state.set_line_width(w.as_f64()?);
        Ok(None)
    }

    // J
    fn set_line_cap(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let w = operation.operand(0)?;
        self.cur_state.path_state.set_line_cap(w.as_i64()?);
        Ok(None)
    }

    // j
    fn set_line_join(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let j = operation.operand(0)?;
        self.cur_state.path_state.set_line_join(j.as_i64()?);
        Ok(None)
    }

    // q
    fn push_graph_state(&mut self) -> PDFResult<Option<GraphicsObject>> {
        self.state_stack.push(self.cur_state.clone());
        Ok(None)
    }

    // Q
    fn pop_graph_state(&mut self) -> PDFResult<Option<GraphicsObject>> {
        if let Some(state) = self.state_stack.pop() {
            self.cur_state = state;
        } else {
            warn!("State stack is empty!");
        }
        Ok(None)
    }

    //cm
    fn modify_current_transform_matrix(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        let a = operation.operand(0)?.as_f64()?;
        let b = operation.operand(1)?.as_f64()?;
        let c = operation.operand(2)?.as_f64()?;
        let d = operation.operand(3)?.as_f64()?;
        let e = operation.operand(4)?.as_f64()?;
        let f = operation.operand(5)?.as_f64()?;
        self.cur_state
            .update_ctm_matrix(&Matrix::new(a, b, c, d, e, f));
        Ok(None)
    }

    // Do
    fn do_operation(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let xobject_name = operation.operand(0)?.as_string()?;
        let xobject = self
            .resource
            .as_ref()
            .unwrap()
            .get_value("XObject")
            .unwrap();
        let xob = match xobject {
            PDFObject::Indirect(_) => self.doc.read_indirect(xobject)?,
            PDFObject::Dictionary(_) => xobject.to_owned(),
            _ => {
                return Err(PDFError::InvalidSyntax("xobjects not exist".to_string()));
            }
        };
        if let Some(obj) = xob.get_value(xobject_name.as_str()) {
            let obj_stream = self.doc.get_object_without_indriect(obj)?;
            let obj_type = obj_stream.get_value_as_string("Subtype").unwrap()?;
            match obj_type.as_str() {
                "Image" => {
                    let width = obj_stream.get_value("Width").unwrap().as_f64()?;
                    let color_space = match obj_stream.get_value("ColorSpace") {
                        Some(sc) => {
                            let cos = self.doc.get_object_without_indriect(sc)?;
                            println!("{:?}", cos);
                            //Some(cos.as_string()?)
                            None
                        }
                        None => None,
                    };
                    let height = obj_stream.get_value("Height").unwrap().as_f64()?;
                    let bits_per_component =
                        obj_stream.get_value("BitsPerComponent").unwrap().as_u32()?;
                    let bytes = obj_stream.bytes()?;
                    let image = Image::new(
                        width,
                        height,
                        color_space,
                        bits_per_component,
                        bytes,
                        self.cur_state.clone(),
                    );
                    return Ok(Some(GraphicsObject::Image(image)));
                }
                "Form" => {
                    // TODO
                    println!("form object");
                }
                _ => {}
            }
        }
        Ok(None)
    }

    // BMC
    fn begin_marked_content(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }
    // BDC
    fn begin_marked_content_dictionary(
        &mut self,
        _operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }
    // EMC
    fn end_marked_content(&mut self, _operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        Ok(None)
    }

    // Text operation
    // BT
    fn begin_text(&mut self) -> PDFResult<Option<GraphicsObject>> {
        self.cur_state.text_state.set_text_matrix(Matrix::default());
        self.cur_state
            .text_state
            .set_text_line_matrix(Matrix::default());
        // self.device.borrow_mut().start_text();
        Ok(None)
    }
    // ET
    fn end_text(&mut self) -> PDFResult<Option<GraphicsObject>> {
        // self.device.borrow_mut().end_text();
        Ok(None)
    }

    // Tc
    fn set_text_character_spacing(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        let char_spacing = operation.operand(0)?.as_f64()?;
        self.cur_state.text_state.set_char_space(char_spacing);
        Ok(None)
    }

    // Tw
    fn set_text_word_spacing(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let word_spacing = operation.operand(0)?.as_f64()?;
        self.cur_state.text_state.set_word_space(word_spacing);
        Ok(None)
    }

    // Tz
    fn set_text_horizal_scaling(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        let scale = operation.operand(0)?.as_f64()?;
        self.cur_state.text_state.set_text_horz_scale(scale);
        Ok(None)
    }
    // TL
    fn set_text_leading(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let leading = operation.operand(0)?.as_f64()?;
        self.cur_state.text_state.set_text_leading(leading);
        Ok(None)
    }

    // Tf
    fn set_text_font(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        //TODO
        let fontname = operation.operand(0)?.as_string()?;
        let size = operation.operand(1)?.as_i64()? as f64;
        let font = self.page.get_font(&fontname)?;
        self.cur_state.text_state.set_font(font);
        self.cur_state.text_state.set_font_size(size);
        Ok(None)
    }

    // Tr
    fn set_text_reander_mode(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let mode = operation.operand(0)?.as_i64()?;
        self.cur_state.text_state.set_render_mode(mode);
        Ok(None)
    }

    // Ts
    fn set_text_rise_mode(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let rise = operation.operand(0)?.as_f64()?;
        self.cur_state.text_state.set_text_rise(rise);
        Ok(None)
    }

    // Td
    fn text_move_start_next_line(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        let x = operation.operand(0)?.as_f64()?;
        let y = operation.operand(1)?.as_f64()?;
        let mat = Matrix::new_translation_matrix(x, y);
        let tm = mat.mutiply(self.cur_state.text_state.text_line_matrix());
        self.cur_state.text_state.set_text_matrix(tm.clone());
        self.cur_state.text_state.set_text_line_matrix(tm);
        Ok(None)
    }
    // TD
    fn text_move_start_next_line_with_leading(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        let ty = operation.operand(1)?.as_f64()? * -1.0;
        let tlop = Operation::new(
            "TL".to_string(),
            vec![PDFObject::Number(PDFNumber::Real(ty))],
        );
        self.set_text_leading(tlop)?;
        self.text_move_start_next_line(operation)?;
        Ok(None)
    }

    // Tm
    fn text_set_text_matrix(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let a = operation.operand(0)?.as_f64()?;
        let b = operation.operand(1)?.as_f64()?;
        let c = operation.operand(2)?.as_f64()?;
        let d = operation.operand(3)?.as_f64()?;
        let e = operation.operand(4)?.as_f64()?;
        let f = operation.operand(5)?.as_f64()?;
        let matrix = Matrix::new(a, b, c, d, e, f);
        self.cur_state
            .text_state
            .set_text_line_matrix(matrix.clone());
        self.cur_state.text_state.set_text_matrix(matrix);
        Ok(None)
    }

    // T*
    fn text_set_move_next_line(&mut self) -> PDFResult<Option<GraphicsObject>> {
        let leading = self.cur_state.text_state.text_leading();
        let op = Operation::new(
            "Td".to_string(),
            vec![
                PDFObject::Number(PDFNumber::Real(0.0)),
                PDFObject::Number(PDFNumber::Real(-1.0 * leading)),
            ],
        );
        self.text_move_start_next_line(op)
    }

    // Tj
    fn show_text(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let content: PDFString = operation.operand(0)?.to_owned().try_into()?;
        let bytes = content.binary_bytes()?;
        let text_codes = TextOpItem::new(bytes, None);
        let obj = Text::new(vec![text_codes], self.cur_state.clone());
        let matrix = obj.get_text_matrix();
        self.cur_state.text_state.set_text_matrix(matrix);
        Ok(Some(GraphicsObject::Text(obj)))
    }

    // "'"
    fn move_next_line_and_show_text(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        self.text_set_move_next_line()?;
        self.show_text(operation)
    }

    // "
    fn move_text_line_and_show_text_with_leading(
        &mut self,
        operation: Operation,
    ) -> PDFResult<Option<GraphicsObject>> {
        let aw = operation.operand(0)?.as_f64()?;
        let ac = operation.operand(1)?.as_f64()?;
        let content = operation.operand(2)?.to_owned();
        //
        self.set_text_character_spacing(Operation::new(
            "Tc".to_string(),
            vec![PDFObject::Number(PDFNumber::Real(ac))],
        ))?;

        self.set_text_word_spacing(Operation::new(
            "Tw".to_string(),
            vec![PDFObject::Number(PDFNumber::Real(aw))],
        ))?;

        self.show_text(Operation::new("Tj".to_string(), vec![content]))
    }

    // TJ
    fn show_text_array(&mut self, operation: Operation) -> PDFResult<Option<GraphicsObject>> {
        let params = operation.operand(0)?.as_array()?;
        let mut pos: Option<f64> = None;
        let mut contents: Vec<TextOpItem> = Vec::new();

        for operand in params {
            match operand {
                PDFObject::String(s) => {
                    let item = TextOpItem::new(s.binary_bytes()?, pos);
                    contents.push(item);
                    pos = None;
                }
                PDFObject::Number(v) => {
                    pos = Some(v.as_f64());
                }
                _ => {
                    return Err(PDFError::InvalidSyntax(format!(
                        "TJ impossiable:{:?}",
                        operand
                    )));
                }
            }
        }
        let text_obj = Text::new(contents, self.cur_state.clone());
        let text_matrix = text_obj.get_text_matrix();
        self.cur_state.text_state.set_text_matrix(text_matrix);
        Ok(Some(GraphicsObject::Text(text_obj)))
    }
}
