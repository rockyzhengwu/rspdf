use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::rc::Rc;

use log::{debug, warn};

use crate::device::Device;
use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::Font;
use crate::geom::matrix::Matrix;
use crate::geom::path::Path;
use crate::geom::point::Point;
use crate::geom::rectangle::Rectangle;
use crate::object::{PDFNumber, PDFObject};
use crate::page::content_parser::ContentParser;
use crate::page::graphics_state::GraphicsState;
use crate::page::operation::Operation;
use crate::page::text::{PageText, TextItem};
use crate::page::Page;

pub struct ContentInterpreter<'a, T: Seek + Read, D: Device> {
    doc: &'a Document<T>,
    page: &'a Page<'a, T>,
    parser: ContentParser,
    state_stack: Vec<GraphicsState>,
    resource_stack: Vec<PDFObject>,

    current_path: Path,
    text_matrix: Matrix,
    text_line_matrix: Matrix,
    font_cache: HashMap<String, Font>,
    device: Rc<RefCell<D>>,
}

impl<'a, T: Seek + Read, D: Device> ContentInterpreter<'a, T, D> {
    pub fn try_new(
        page: &'a Page<'_, T>,
        doc: &'a Document<T>,
        device: Rc<RefCell<D>>,
    ) -> PDFResult<Self> {
        let contents = page.contents()?;
        let mut buffers = Vec::new();
        for content in contents {
            buffers.extend(content.bytes());
        }
        let sss = String::from_utf8_lossy(buffers.as_slice());
        for item in sss.split('\n') {
            // println!("{:?}", item);
        }
        let parser = ContentParser::try_new(buffers)?;
        Ok(ContentInterpreter {
            page,
            doc,
            parser,
            state_stack: Vec::new(),
            resource_stack: Vec::new(),
            current_path: Path::default(),
            text_matrix: Matrix::default(),
            text_line_matrix: Matrix::default(),
            font_cache: HashMap::new(),
            device,
        })
    }

    pub fn run(&mut self) -> PDFResult<()> {
        let media = self.page.media_bbox()?;
        let crop = self.page.crop_bbox()?;
        self.device
            .borrow_mut()
            .begain_page(&self.page.number, media, crop);
        let resource = self.page.resources()?;
        let state = GraphicsState::default();
        self.resource_stack.push(resource);
        self.state_stack.push(state);

        while let Ok(op) = self.parser.parse_operation() {
            // debug!("{:?}", op);
            self.invoke_operation(op)?;
        }
        self.device.borrow_mut().end_page(&self.page.number);
        Ok(())
    }

    fn invoke_operation(&mut self, operation: Operation) -> PDFResult<()> {
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
            "cs" => self.set_color_space(operation),
            "CS" => self.set_color_space_fill(operation),
            "sc" | "scn" => self.set_color_fill(operation),
            "SC" | "SCN" => self.set_color_stroke(operation),
            "gs" => self.set_graphstate_resource(operation),
            // Image
            "EI" => self.end_image(operation),

            _ => Ok(()),
        }
    }

    // EI
    fn end_image(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }

    // scn
    fn set_graphstate_resource(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }
    // SC
    fn set_color_stroke(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }
    // sc
    fn set_color_fill(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }

    // CS
    fn set_color_space_fill(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }
    // cs
    fn set_color_space(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }
    // K
    fn set_cmyk_stroke(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }
    // k
    fn set_cmyk_fill(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }
    //RG
    fn set_rgb_stroke(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }
    // rg
    fn set_rgb_fill(&mut self, _operation: Operation) -> PDFResult<()> {
        //pass
        Ok(())
    }
    // G
    fn set_gray_stroke(&mut self, _operation: Operation) -> PDFResult<()> {
        //pass
        Ok(())
    }
    // g
    fn set_gray_fill(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }

    // S
    fn paint_path(&mut self, _operation: Operation) -> PDFResult<()> {
        // TODO implement paint path
        let state = self.last_mut_state().clone();
        //self.current_path.close_last_subpath();
        let mut path = std::mem::take(&mut self.current_path);
        path.close_last_subpath();
        Ok(())
    }

    // re
    fn reanctle(&mut self, operation: Operation) -> PDFResult<()> {
        let lx = operation.operand(0)?.as_f64()?;
        let ly = operation.operand(1)?.as_f64()?;
        let ux = operation.operand(2)?.as_f64()?;
        let uy = operation.operand(3)?.as_f64()?;
        self.current_path.rectangle(Rectangle::new(lx, ly, ux, uy));

        Ok(())
    }

    // h
    fn close_sub_path(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }

    // y
    fn curve_fourh_point_duplicate(&mut self, operation: Operation) -> PDFResult<()> {
        let x1 = operation.operand(0)?.as_f64()?;
        let y1 = operation.operand(0)?.as_f64()?;
        let x3 = operation.operand(0)?.as_f64()?;
        let y3 = operation.operand(0)?.as_f64()?;

        let p1 = Point::new(x1, y1);
        let p3 = Point::new(x3, y3);
        self.current_path.curve_to(vec![p1, p3.clone(), p3.clone()]);
        Ok(())
    }

    // v
    fn curve_first_point_duplicate(&mut self, operation: Operation) -> PDFResult<()> {
        let x2 = operation.operand(0)?.as_f64()?;
        let y2 = operation.operand(0)?.as_f64()?;
        let x3 = operation.operand(0)?.as_f64()?;
        let y3 = operation.operand(0)?.as_f64()?;

        let p2 = Point::new(x2, y2);
        let p3 = Point::new(x3, y3);
        self.current_path.curve_to(vec![p2, p3]);
        Ok(())
    }

    // c
    fn curve(&mut self, operation: Operation) -> PDFResult<()> {
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

        Ok(())
    }

    // l lineto
    fn line_to(&mut self, operation: Operation) -> PDFResult<()> {
        let x = operation.operand(0)?.as_f64()?;
        let y = operation.operand(1)?.as_f64()?;
        let p = Point::new(x, y);
        self.current_path.line_to(p);
        Ok(())
    }

    // m moveto
    fn move_to(&mut self, operation: Operation) -> PDFResult<()> {
        let x = operation.operand(0)?.as_f64()?;
        let y = operation.operand(1)?.as_f64()?;
        let p = Point::new(x, y);
        self.current_path.move_to(p);
        Ok(())
    }

    // d
    fn set_line_dash_pattern(&mut self, _operation: Operation) -> PDFResult<()> {
        //let pattern = operation.operands.get(0).unwrap();
        //self.last_mut_state().set_line_dash_pattern(pattern);
        Ok(())
    }

    // M
    fn set_miter_limit(&mut self, operation: Operation) -> PDFResult<()> {
        let limit = operation.operand(0)?;
        self.last_mut_state().set_line_miter_limit(limit.as_i64()?);
        Ok(())
    }

    // w
    fn set_line_width(&mut self, operation: Operation) -> PDFResult<()> {
        let w = operation.operand(0)?;
        self.last_mut_state().set_line_width(w.as_f64()?);
        Ok(())
    }

    // J
    fn set_line_cap(&mut self, operation: Operation) -> PDFResult<()> {
        let w = operation.operand(0)?;
        self.last_mut_state().set_line_cap_style(w.as_i64()?);
        Ok(())
    }

    // j
    fn set_line_join(&mut self, operation: Operation) -> PDFResult<()> {
        let j = operation.operand(0)?;
        self.last_mut_state().set_line_join(j.as_i64()?);
        Ok(())
    }

    fn last_mut_state(&mut self) -> &mut GraphicsState {
        self.state_stack.last_mut().unwrap()
    }
    fn last_state(&self) -> &GraphicsState {
        self.state_stack.last().unwrap()
    }

    fn display_string(&mut self, content: &PDFObject) -> PDFResult<()> {
        let state = self.last_state();
        let fontname = state.font();
        let font = self.page.get_font(fontname)?;
        match content {
            PDFObject::String(s) => {
                let bytes = s.binary_bytes()?;
                let chars = font.decode_charcodes(&bytes);
                let mut texts = Vec::new();
                let mut text_matrix = self.text_matrix.clone();
                for ch in chars {
                    let width =
                        font.get_width(ch.cid()) * 0.001 * state.font_size() + state.char_spacing();
                    let text_item = TextItem::new(
                        text_matrix.clone(),
                        ch.unicode().to_owned(),
                        ch.cid().to_owned(),
                    );
                    let mat = Matrix::new_translation_matrix(width, 0.0);
                    text_matrix = mat.mutiply(&text_matrix);
                    texts.push(text_item);
                }
                let textobj = PageText::new(texts, &font, state.font_size(), state.ctm().clone());
                self.device.borrow_mut().show_text(&textobj)?;
                self.text_matrix = text_matrix;
                Ok(())
            }
            _ => Err(PDFError::ContentInterpret(format!(
                "Display content need to be PDFString got:{:?}",
                content
            ))),
        }
    }

    // q
    fn push_graph_state(&mut self) -> PDFResult<()> {
        let last = self.state_stack.last().unwrap().clone();
        self.state_stack.push(last);
        Ok(())
    }

    // Q
    fn pop_graph_state(&mut self) -> PDFResult<()> {
        self.state_stack.pop();
        Ok(())
    }

    //cm
    fn modify_current_transform_matrix(&mut self, operation: Operation) -> PDFResult<()> {
        let a = operation.operand(0)?.as_f64()?;
        let b = operation.operand(1)?.as_f64()?;
        let c = operation.operand(2)?.as_f64()?;
        let d = operation.operand(3)?.as_f64()?;
        let e = operation.operand(4)?.as_f64()?;
        let f = operation.operand(5)?.as_f64()?;
        self.last_mut_state()
            .update_ctm_matrix(&Matrix::new(a, b, c, d, e, f));
        Ok(())
    }

    // Do
    fn do_operation(&mut self, operation: Operation) -> PDFResult<()> {
        let xobject_name = operation.operand(0)?.as_string()?;
        let xobject = self
            .resource_stack
            .last()
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
        let obj = xob.get_value(xobject_name.as_str()).unwrap();
        let _obj_stream = self.doc.read_indirect(obj)?;
        Ok(())
    }

    // BMC
    fn begin_marked_content(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }
    // BDC
    fn begin_marked_content_dictionary(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }
    // EMC
    fn end_marked_content(&mut self, _operation: Operation) -> PDFResult<()> {
        Ok(())
    }

    // Text operation
    // BT
    fn begin_text(&mut self) -> PDFResult<()> {
        self.text_matrix = Matrix::default();
        self.text_line_matrix = Matrix::default();
        self.device.borrow_mut().start_text();
        Ok(())
    }
    // ET
    fn end_text(&mut self) -> PDFResult<()> {
        self.device.borrow_mut().end_text();
        Ok(())
    }

    // Tc
    fn set_text_character_spacing(&mut self, operation: Operation) -> PDFResult<()> {
        // TODO error
        let char_spacing = operation.operand(0)?.as_f64()?;
        println!("Tc {:?}", char_spacing);
        let state = self.last_mut_state();
        state.set_char_spacing(char_spacing);
        Ok(())
    }

    // Tw
    fn set_text_word_spacing(&mut self, operation: Operation) -> PDFResult<()> {
        // TODO error
        let word_spacing = operation.operand(0)?.as_f64()?;
        self.last_mut_state().set_word_spacing(word_spacing);
        Ok(())
    }

    // Tz
    fn set_text_horizal_scaling(&mut self, operation: Operation) -> PDFResult<()> {
        let scale = operation.operand(0)?.as_f64()?;
        self.last_mut_state().set_hscaling(scale);
        Ok(())
    }
    // TL
    fn set_text_leading(&mut self, operation: Operation) -> PDFResult<()> {
        let leading = operation.operand(0)?.as_f64()?;
        self.last_mut_state().set_text_leading(leading);
        Ok(())
    }

    // Tf
    fn set_text_font(&mut self, operation: Operation) -> PDFResult<()> {
        //TODO
        let fontname = operation.operand(0)?.as_string()?;
        let size = operation.operand(1)?.as_i64()? as f64;
        self.last_mut_state().set_font(fontname, size);
        Ok(())
    }

    // Tr
    fn set_text_reander_mode(&mut self, operation: Operation) -> PDFResult<()> {
        let render = operation.operand(0)?.as_i64()?;
        let state = self.state_stack.last_mut().unwrap();
        state.set_rendering_indent(render);
        Ok(())
    }

    // Ts
    fn set_text_rise_mode(&mut self, operation: Operation) -> PDFResult<()> {
        let rise = operation.operand(0)?.as_f64()?;
        self.last_mut_state().set_text_rise(rise);
        Ok(())
    }

    // Td
    fn text_move_start_next_line(&mut self, operation: Operation) -> PDFResult<()> {
        let x = operation.operand(0)?.as_f64()?;
        let y = operation.operand(1)?.as_f64()?;
        let mat = Matrix::new_translation_matrix(x, y);
        self.text_matrix = mat.mutiply(&self.text_line_matrix);
        self.text_line_matrix = self.text_matrix.clone();
        Ok(())
    }
    // TD
    fn text_move_start_next_line_with_leading(&mut self, operation: Operation) -> PDFResult<()> {
        let ty = operation.operand(1)?.as_f64()? * -1.0;
        let tlop = Operation::new(
            "TL".to_string(),
            vec![PDFObject::Number(PDFNumber::Real(ty))],
        );
        self.set_text_leading(tlop)?;
        self.text_move_start_next_line(operation)
    }

    // Tm
    fn text_set_text_matrix(&mut self, operation: Operation) -> PDFResult<()> {
        let a = operation.operand(0)?.as_f64()?;
        let b = operation.operand(1)?.as_f64()?;
        let c = operation.operand(2)?.as_f64()?;
        let d = operation.operand(3)?.as_f64()?;
        let e = operation.operand(4)?.as_f64()?;
        let f = operation.operand(5)?.as_f64()?;
        let matrix = Matrix::new(a, b, c, d, e, f);
        self.text_line_matrix = matrix.clone();
        self.text_matrix = matrix;
        Ok(())
    }

    // T*
    fn text_set_move_next_line(&mut self) -> PDFResult<()> {
        let leading = self.last_mut_state().text_leading();
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
    fn show_text(&mut self, operation: Operation) -> PDFResult<()> {
        let content = &operation.operand(0)?;
        self.display_string(content)?;
        Ok(())
    }

    // "'"
    fn move_next_line_and_show_text(&mut self, operation: Operation) -> PDFResult<()> {
        self.text_set_move_next_line()?;
        self.show_text(operation)
    }

    // "
    fn move_text_line_and_show_text_with_leading(&mut self, operation: Operation) -> PDFResult<()> {
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
    fn show_text_array(&mut self, operation: Operation) -> PDFResult<()> {
        let params = operation.operand(0)?.as_array()?;
        for operand in params {
            match operand {
                PDFObject::String(_) => {
                    self.display_string(operand)?;
                }
                PDFObject::Number(v) => {
                    let state = self.last_mut_state();
                    let adjust_by = -1.0 * v.as_f64() * 0.001 * state.font_size();
                    // TODO when hscaling setted adjust
                    if state.hscaling() > 0.0 {
                        warn!("hscaling {:?}", state.hscaling());
                    }
                    let mat = Matrix::new_translation_matrix(adjust_by, 0.0);

                    self.text_matrix = mat.mutiply(&self.text_matrix);
                }
                _ => {
                    return Err(PDFError::InvalidSyntax(format!(
                        "TJ impossiable:{:?}",
                        operand
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.state_stack.clear();
        self.resource_stack.clear();
        self.current_path = Path::default();
        self.text_line_matrix = Matrix::default();
        self.text_matrix = Matrix::default();
        self.font_cache = HashMap::new();
    }
}
