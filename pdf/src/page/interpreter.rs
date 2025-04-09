use log::{debug, error, warn};

use super::graphics_state::FillRule;
use crate::{
    color::{
        device_cmyk::DeviceCmyk, device_gray::DeviceGray, device_rgb::DeviceRgb, parse_colorspace,
        pattern::Pattern, value::ColorValue, ColorSpace,
    },
    device::Device,
    error::{PdfError, Result},
    font::{pdf_font::Font, WritingMode},
    geom::{
        coordinate::{Matrix, Point},
        path::Path,
        rect::Rect,
    },
    object::{number::PdfNumber, stream::PdfStream, PdfObject},
    page::{
        content_parser::ContentParser, graphics_state::GraphicsState, image::PdfImage,
        operator::Operator, resource::Resources, Page,
    },
    xref::Xref,
};

pub struct Interpreter<'a> {
    page: &'a Page<'a>,
    xref: &'a Xref,
    state: GraphicsState,
    parser: ContentParser,
    state_stack: Vec<GraphicsState>,
    current_path: Option<Path>,
    current_point: Option<Point>,
    resources: Vec<Resources>,
}

impl<'a> Interpreter<'a> {
    pub fn try_new(page: &'a Page, xref: &'a Xref) -> Result<Self> {
        let contents = page.content_stream()?;
        let mut buffer = Vec::new();
        for content in contents {
            let bytes = content.decode_data(Some(xref))?;
            buffer.extend(bytes);
        }
        let sss = String::from_utf8_lossy(buffer.as_slice());
        let parser = ContentParser::new(buffer);
        let mut resources = Vec::new();
        resources.push(page.resources().to_owned());
        Ok(Interpreter {
            page,
            xref,
            state: GraphicsState::default(),
            parser,
            state_stack: Vec::new(),
            current_path: None,
            current_point: None,
            resources,
        })
    }

    pub fn run(&mut self, num: u32, device: &mut dyn Device) -> Result<()> {
        let bbox = self.page.mediabox()?;

        let rotate = self.page.rotated()? % 360;
        let kx = device.hdpi() / 72.0;
        let ky = device.vdpi() / 72.0;

        let (page_width, page_height) = match rotate {
            0 => {
                self.state.ctm = Matrix::new(kx, 0.0, 0.0, -ky, kx * bbox.lx(), ky * bbox.uy());
                let page_width = kx * bbox.width();
                let page_height = ky * bbox.height();
                (page_width, page_height)
            }
            90 => {
                self.state.ctm = Matrix::new(0.0, ky, kx, 0.0, -ky * bbox.ly(), -ky * bbox.lx());
                let page_width = kx * bbox.height();
                let page_height = ky * bbox.width();
                (page_width, page_height)
            }
            180 => {
                self.state.ctm = Matrix::new(-kx, 0.0, 0.0, ky, kx * bbox.ux(), -ky * bbox.ly());
                let page_width = kx * bbox.width();
                let page_height = ky * bbox.height();
                (page_width, page_height)
            }
            270 => {
                self.state.ctm = Matrix::new(0.0, -ky, -kx, 0.0, kx * bbox.uy(), ky * bbox.ux());
                let page_width = kx * bbox.height();
                let page_height = ky * bbox.width();
                (page_width, page_height)
            }
            _ => {
                return Err(PdfError::Interpreter("Invalid rotate of page".to_string()));
            }
        };

        let _userunit = self.page.user_unit();

        device.start_page(&self.state, num, page_width, page_height)?;
        if let Some(cropbox) = self.page.cropbox()? {
            self.state.clipping_path.rect(cropbox);
            device.clip(&self.state)?;
        }

        while let Ok(op) = self.parser.read_operator() {
            match self.invoke_operator(op, device) {
                Ok(()) => {
                    //
                }
                Err(e) => {
                    println!("{:?}", e);
                    error!("Operator error:{:?}", e);
                }
            }
        }
        device.end_page(&self.state)?;
        Ok(())
    }

    // q
    fn push_graph_state(&mut self) -> Result<()> {
        self.state_stack.push(self.state.clone());
        Ok(())
    }

    // Q
    fn pop_graph_state(&mut self) -> Result<()> {
        if let Some(state) = self.state_stack.pop() {
            self.state = state;
        } else {
            return Err(PdfError::Interpreter("State stack is empty".to_string()));
        }
        Ok(())
    }

    // cm
    fn modify_current_transform_matrix(
        &mut self,
        op: Operator,
        device: &mut dyn Device,
    ) -> Result<()> {
        let a = op.operand(0)?.as_number()?.real();
        let b = op.operand(1)?.as_number()?.real();
        let c = op.operand(2)?.as_number()?.real();
        let d = op.operand(3)?.as_number()?.real();
        let e = op.operand(4)?.as_number()?.real();
        let f = op.operand(5)?.as_number()?.real();
        self.state.update_ctm_matrix(&Matrix::new(a, b, c, d, e, f));
        Ok(())
    }

    fn do_form(&mut self, xobject: &PdfStream, device: &mut dyn Device) -> Result<()> {
        if let Some(res) = xobject.get_from_dict("Resources") {
            match res {
                PdfObject::Indirect(_) => {
                    let res = self.xref.read_object(res)?.to_dict()?;
                    let resources = Resources::try_new(&res, self.xref)?;
                    self.resources.push(resources);
                }
                PdfObject::Dict(d) => {
                    let resources = Resources::try_new(d, self.xref)?;
                    self.resources.push(resources);
                }
                _ => {
                    return Err(PdfError::Interpreter(
                        "Form xojbect resources is invalid".to_string(),
                    ));
                }
            }
        }

        self.push_graph_state()?;
        if let Some(mat) = xobject.get_from_dict("Matrix") {
            let mat = mat.as_array().map_err(|e| {
                PdfError::Interpreter("Form xobject Matrix is not an array".to_string())
            })?;

            let a = mat
                .get(0)
                .ok_or(PdfError::Interpreter(
                    "Form Matrix array element error".to_string(),
                ))?
                .as_number()?
                .real();

            let b = mat
                .get(1)
                .ok_or(PdfError::Interpreter(
                    "Form Matrix array element error".to_string(),
                ))?
                .as_number()?
                .real();

            let c = mat
                .get(2)
                .ok_or(PdfError::Interpreter(
                    "Form Matrix array element error".to_string(),
                ))?
                .as_number()?
                .real();

            let d = mat
                .get(3)
                .ok_or(PdfError::Interpreter(
                    "Form Matrix array element error".to_string(),
                ))?
                .as_number()?
                .real();

            let e = mat
                .get(4)
                .ok_or(PdfError::Interpreter(
                    "Form Matrix array element error".to_string(),
                ))?
                .as_number()?
                .real();
            let f = mat
                .get(5)
                .ok_or(PdfError::Interpreter(
                    "Form Matrix array element error".to_string(),
                ))?
                .as_number()?
                .real();
            let fm = Matrix::new(a, b, c, d, e, f);
            self.state.update_ctm_matrix(&fm);
        }
        // TODO bbox as clip path

        let form_data = xobject.decode_data(Some(self.xref))?;
        //println!("{:?}", String::from_utf8(form_data.clone()));
        let parser = ContentParser::new(form_data);
        while let Ok(op) = parser.read_operator() {
            self.invoke_operator(op, device)?;
        }

        self.pop_graph_state()?;
        self.resources.pop();
        Ok(())
    }

    // do
    fn do_operation(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        let xobject_name = op.operand(0)?.as_name()?.name();
        if let Some(xobject) = self.current_resource()?.lookup_xobject(xobject_name) {
            let xobject = self.xref.read_object(xobject)?;
            let xt = xobject
                .get_from_dict("Subtype")
                .ok_or(PdfError::Interpreter("XObject Subtype is None".to_string()))?
                .as_name()?;
            match xt.name() {
                "Image" => {
                    let image_stream = xobject.as_stream().map_err(|_| {
                        PdfError::Interpreter("Xobject Image need a stream".to_string())
                    })?;
                    let pdf_image = PdfImage::try_new(image_stream, self.xref)?;
                    device.draw_image(pdf_image, &self.state)?;
                }
                "Form" => {
                    let xs = xobject.as_stream().map_err(|_| {
                        PdfError::Interpreter("Form object is not a stream".to_string())
                    })?;
                    self.do_form(xs, device)?;
                    return Ok(());
                }
                _ => {
                    return Err(PdfError::Interpreter(format!(
                        "XObject Subtype must be Image or Form got :{:?}",
                        xt
                    )));
                }
            }
        } else {
            warn!("DO {:?} not found", xobject_name);
        }
        Ok(())
    }

    // BMC
    fn begin_marked_content(&mut self, op: Operator) -> Result<()> {
        Ok(())
    }

    // BDC
    fn begin_marked_content_dictionary(&mut self, op: Operator) -> Result<()> {
        Ok(())
    }

    fn end_marked_content(&mut self, op: Operator) -> Result<()> {
        Ok(())
    }

    //w
    fn set_line_width(&mut self, op: Operator) -> Result<()> {
        let width = op
            .operand(0)
            .map_err(|_| {
                PdfError::Interpreter(format!("Operator set line width paramater error:{:?}", op))
            })?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!("Operator set linewidth need a number got:{:?}", op))
            })?
            .real();
        self.state.line_width = width;
        Ok(())
    }
    // J
    fn set_line_cap(&mut self, op: Operator) -> Result<()> {
        let c = op.operand(0)?.as_number()?.integer();
        self.state.set_line_cap(c);

        Ok(())
    }

    // j
    fn set_line_join(&mut self, op: Operator) -> Result<()> {
        let j = op.operand(0)?.as_number()?.integer();
        self.state.set_line_join(j);
        Ok(())
    }

    // M
    fn set_miter_limit(&mut self, op: Operator) -> Result<()> {
        let m = op.operand(0)?.as_number()?.real();
        self.state.miter_limit = m;
        Ok(())
    }

    // d
    fn set_line_dash_pattern(&mut self, op: Operator) -> Result<()> {
        let pattern = op.operand(0)?.as_array()?;
        let mut dash_array = Vec::new();
        for v in pattern.iter() {
            dash_array.push(v.integer()? as u32);
        }
        let dash_phase = op.operand(1)?.integer()? as u32;
        self.state.set_dash_pattern(dash_array, dash_phase);
        Ok(())
    }

    // ri
    fn set_render_intent(&mut self, op: Operator) -> Result<()> {
        let intent = op.operand(0)?.as_name()?.name();
        self.state.set_render_intent(intent);
        Ok(())
    }

    // i
    fn set_flatness(&mut self, op: Operator) -> Result<()> {
        let flatness = op.operand(0)?.integer()?;
        self.state.flatness = flatness;
        Ok(())
    }

    // m
    fn move_to(&mut self, op: Operator) -> Result<()> {
        let x = op.operand(0)?.as_number()?.real();
        let y = op.operand(1)?.as_number()?.real();
        let point = Point::new(x, y);
        self.current_point = Some(point.clone());
        match &mut self.current_path {
            Some(p) => {
                p.move_to(point.clone());
                self.current_point = Some(point)
            }
            None => {
                let mut p = Path::default();
                p.move_to(point);
                self.current_path = Some(p);
            }
        }
        Ok(())
    }

    // l
    fn line_to(&mut self, op: Operator) -> Result<()> {
        let x = op.operand(0)?.as_number()?.real();
        let y = op.operand(1)?.as_number()?.real();
        let p = Point::new(x, y);
        match self.current_path.as_mut() {
            Some(path) => {
                path.line_to(p.clone())?;
                self.current_point = Some(p);
            }
            None => {
                return Err(PdfError::Interpreter(
                    "Lineto Current point is Null".to_string(),
                ))
            }
        }
        Ok(())
    }

    // c
    fn curve_to(&mut self, op: Operator) -> Result<()> {
        let x1 = op.operand(0)?.as_number()?.real();
        let y1 = op.operand(1)?.as_number()?.real();
        let x2 = op.operand(2)?.as_number()?.real();
        let y2 = op.operand(3)?.as_number()?.real();
        let x3 = op.operand(4)?.as_number()?.real();
        let y3 = op.operand(5)?.as_number()?.real();
        let p1 = Point::new(x1, y1);
        let p2 = Point::new(x2, y2);
        let p3 = Point::new(x3, y3);
        let p0 = self.current_point.clone().ok_or(PdfError::Interpreter(
            "c operator current_point is None".to_string(),
        ))?;
        match self.current_path.as_mut() {
            Some(path) => {
                path.curve4(p0, p1, p2, p3.clone())?;
                self.current_point = Some(p3)
            }
            None => {
                return Err(PdfError::Interpreter(
                    "Curve Current point is Null".to_string(),
                ))
            }
        }
        Ok(())
    }

    // v
    fn curve_first_point_duplicate(&mut self, op: Operator) -> Result<()> {
        let x2 = op.operand(0)?.as_number()?.real();
        let y2 = op.operand(1)?.as_number()?.real();
        let x3 = op.operand(2)?.as_number()?.real();
        let y3 = op.operand(3)?.as_number()?.real();

        let p2 = Point::new(x2, y2);
        let p3 = Point::new(x3, y3);

        let p0 = self.current_point.clone().ok_or(PdfError::Interpreter(
            "c operator current_point is None".to_string(),
        ))?;
        match self.current_path.as_mut() {
            Some(path) => {
                path.curve3(p0, p2, p3.clone())?;
                self.current_point = Some(p3)
            }
            None => {
                return Err(PdfError::Interpreter(
                    "v operator Curve Current point is Null".to_string(),
                ))
            }
        }
        Ok(())
    }

    //y
    fn curve_fourth_point_duplicate(&mut self, op: Operator) -> Result<()> {
        let x1 = op.operand(0)?.as_number()?.real();
        let y1 = op.operand(1)?.as_number()?.real();
        let x3 = op.operand(2)?.as_number()?.real();
        let y3 = op.operand(3)?.as_number()?.real();

        let p1 = Point::new(x1, y1);
        let p3 = Point::new(x3, y3);
        let p0 = self.current_point.clone().ok_or(PdfError::Interpreter(
            "c operator current_point is None".to_string(),
        ))?;
        match self.current_path.as_mut() {
            Some(path) => {
                path.curve3(p0, p1, p3.clone())?;
                self.current_point = Some(p3)
            }
            None => {
                return Err(PdfError::Interpreter(
                    "Curve Current point is Null".to_string(),
                ))
            }
        }
        Ok(())
    }

    // re
    fn rectangle(&mut self, op: Operator) -> Result<()> {
        let x = op.operand(0)?.as_number()?.real();
        let y = op.operand(1)?.as_number()?.real();
        let width = op.operand(2)?.as_number()?.real();
        let height = op.operand(3)?.as_number()?.real();
        let rect = Rect::new(Point::new(x, y), width, height);

        match self.current_path.as_mut() {
            Some(path) => {
                path.rect(rect);
            }
            None => {
                self.current_point = Some(rect.lower_left().to_owned());
                let mut path = Path::default();
                path.rect(rect);
                self.current_path = Some(path);
            }
        }
        Ok(())
    }

    // h
    fn close_sub_path(&mut self, _: Operator) -> Result<()> {
        match self.current_path.as_mut() {
            Some(path) => {
                path.close_sub_path()?;
            }
            None => {
                return Err(PdfError::Interpreter(
                    "close_sub_path Current path is None".to_string(),
                ));
            }
        }
        Ok(())
    }
    // n
    fn end_path(&mut self, _: Operator) -> Result<()> {
        // TODO update clipping path
        self.current_point = None;
        self.current_path = None;

        Ok(())
    }
    fn current_resource(&self) -> Result<&Resources> {
        self.resources.last().ok_or(PdfError::Interpreter(
            "Current resources is None".to_string(),
        ))
    }

    //  gs
    fn set_extend_graphic_state(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        let name = op.operand(0)?.as_name()?;
        let ext_state = self
            .current_resource()?
            .lookup_ext_g_state(name.name(), &self.xref)?;
        if let Some(lw) = ext_state.get("LW") {
            let lw = lw.as_number()?.real();
            self.state.line_width = lw;
        }
        if let Some(lc) = ext_state.get("LC") {
            let lc = lc.as_number()?.integer();
            self.state.set_line_cap(lc);
        }
        if let Some(lj) = ext_state.get("LJ") {
            let lj = lj.as_number()?.integer();
            self.state.set_line_join(lj);
        }
        if let Some(ml) = ext_state.get("ML") {
            let ml = ml.as_number()?.real();
            self.state.miter_limit = ml;
        }
        if let Some(dp) = ext_state.get("D") {
            debug!("dash pattern: {:?}", dp);
        }
        if let Some(ri) = ext_state.get("RI") {
            let ri = ri.as_name()?.name();
            self.state.set_render_intent(ri);
        }
        if let Some(op) = ext_state.get("OP") {
            self.state.stroke_overprint = op.as_bool()?.0;
        }
        if let Some(op) = ext_state.get("op") {
            self.state.fill_overprint = op.as_bool()?.0;
        }

        if let Some(opm) = ext_state.get("OPM") {
            self.state.overpint_mode = opm.as_number()?.integer();
        }
        if let Some(font) = ext_state.get("Font") {
            let font = font.as_array()?;
            let fname = font
                .get(0)
                .ok_or(PdfError::Interpreter("ExtState Font bad param".to_string()))?
                .as_name()?
                .name();
            let fsize = font
                .get(1)
                .ok_or(PdfError::Interpreter(
                    "ExtState Fint size bad param".to_string(),
                ))?
                .as_number()?
                .real();
            self.do_set_font(fname, fsize, device)?;
        }

        if let Some(sa) = ext_state.get("SA") {
            self.state.stroke_adjust = sa.as_bool()?.0;
        }
        if let Some(sm) = ext_state.get("SM") {
            self.state.smoothness = sm.as_number()?.real();
        }
        // TODO finish gs dictionary

        Ok(())
    }

    // BT
    fn begin_text(&mut self, device: &mut dyn Device) -> Result<()> {
        self.state.text_matrix = Matrix::default();
        self.state.text_line_matrix = Matrix::default();
        device.begin_text(&self.state)?;
        Ok(())
    }
    // ET
    fn end_text(&mut self, device: &mut dyn Device) -> Result<()> {
        device.end_text(&self.state)?;
        Ok(())
    }
    // Tc
    fn set_text_character_spacing(&mut self, op: Operator) -> Result<()> {
        self.state.char_space = op.operand(0)?.as_number()?.real();
        Ok(())
    }

    // Tw
    fn set_text_word_spacing(&mut self, op: Operator) -> Result<()> {
        self.state.word_space = op.operand(0)?.as_number()?.real();
        Ok(())
    }
    // Tz
    fn set_text_horizal_scaling(&mut self, op: Operator) -> Result<()> {
        self.state.text_horz_scale = op.operand(0)?.as_number()?.real();
        Ok(())
    }

    // tL
    fn set_text_leading(&mut self, op: Operator) -> Result<()> {
        self.state.text_leading = op.operand(0)?.as_number()?.real();
        Ok(())
    }
    // Tf
    fn set_text_font(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        let fname = op.operand(0)?.as_name()?.name();
        let fsize = op.operand(1)?.as_number()?.real();
        self.do_set_font(fname, fsize, device)
    }

    fn do_set_font(&mut self, fname: &str, fsize: f32, device: &mut dyn Device) -> Result<()> {
        let font_dict = self.current_resource()?.lookup_font(fname, &self.xref)?;
        let font = Font::try_new(font_dict, self.xref)?;
        self.state.font = Some(font);
        self.state.font_size = fsize;
        device.update_font(&self.state)?;
        Ok(())
    }

    fn show_text(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        // TODO imple show text
        // 1.update state, text_matrix
        // 2. invoke device show_text method
        let content = op.operand(0)?;
        let codes = match content {
            PdfObject::LiteralString(s) => s.bytes().to_owned(),
            PdfObject::HexString(s) => s.raw_bytes()?,
            _ => {
                return Err(PdfError::Interpreter(
                    "show text operand need be a pdfstring".to_string(),
                ));
            }
        };
        self.do_show_text(codes.as_slice(), device)?;
        Ok(())
    }

    fn do_show_text(&mut self, codes: &[u8], device: &mut dyn Device) -> Result<()> {
        let font = self.state.font.as_ref().ok_or(PdfError::Interpreter(
            "show text current font is None".to_string(),
        ))?;
        let chars = font.chars(codes)?;
        for char in chars.iter() {
            device.draw_char(char, &self.state)?;
            let char_with = char.width();
            let displacement = char_with * 0.001 * self.state.font_size + self.state.char_space;
            let font = self.state.font.as_ref().ok_or(PdfError::Interpreter(
                "show text current font is None".to_string(),
            ))?;
            match font.writting_mode() {
                WritingMode::Horizontal => {
                    let tm = Matrix::new_translation_matrix(displacement, 0.0);
                    let ntm = tm.transform(&self.state.text_matrix);
                    self.state.text_matrix = ntm;
                }
                WritingMode::Vertical => {
                    let tm = Matrix::new_translation_matrix(0.0, displacement);
                    let ntm = tm.transform(&self.state.text_matrix);
                    self.state.text_matrix = ntm;
                }
            }
        }

        Ok(())
    }

    fn set_text_render_mode(&mut self, op: Operator) -> Result<()> {
        let mode = op.operand(0)?.integer()?;
        self.state.set_render_mode(mode);
        Ok(())
    }

    fn set_text_rise_mode(&mut self, op: Operator) -> Result<()> {
        let rise = op.operand(0)?.as_number()?;
        self.state.text_rise = rise.real();
        Ok(())
    }

    fn text_move_start_next_line_with_leading(&mut self, op: Operator) -> Result<()> {
        let ty = op.operand(1)?.as_number()?.real() * -1.0;
        let tlop = Operator::new(
            "TL".to_string(),
            vec![PdfObject::Number(crate::object::number::PdfNumber::Real(
                ty,
            ))],
        );
        self.set_text_leading(tlop)?;
        self.text_move_start_next_line(op)?;
        Ok(())
    }

    fn text_move_start_next_line(&mut self, op: Operator) -> Result<()> {
        let x = op.operand(0)?.as_number()?.real();
        let y = op.operand(1)?.as_number()?.real();
        let mat = Matrix::new_translation_matrix(x, y);
        let tm = mat.transform(&self.state.text_line_matrix);
        self.state.text_matrix = tm.clone();
        self.state.text_line_matrix = tm;
        Ok(())
    }
    fn text_set_text_matrix(&mut self, op: Operator) -> Result<()> {
        let a = op.operand(0)?.as_number()?.real();
        let b = op.operand(1)?.as_number()?.real();
        let c = op.operand(2)?.as_number()?.real();
        let d = op.operand(3)?.as_number()?.real();
        let e = op.operand(4)?.as_number()?.real();
        let f = op.operand(5)?.as_number()?.real();
        let matrix = Matrix::new(a, b, c, d, e, f);
        self.state.text_line_matrix = matrix.clone();
        self.state.text_matrix = matrix;
        Ok(())
    }

    fn text_set_move_next_line(&mut self) -> Result<()> {
        let leading = self.state.text_leading;
        let op = Operator::new(
            "Td".to_string(),
            vec![
                PdfObject::Number(PdfNumber::Real(0.0)),
                PdfObject::Number(PdfNumber::Real(-1.0 * leading)),
            ],
        );
        self.text_move_start_next_line(op)
    }
    fn move_next_line_and_show_text(
        &mut self,
        op: Operator,
        device: &mut dyn Device,
    ) -> Result<()> {
        self.text_set_move_next_line()?;
        self.show_text(op, device)
    }

    fn move_text_line_and_show_text_with_leading(
        &mut self,
        op: Operator,
        device: &mut dyn Device,
    ) -> Result<()> {
        let aw = op.operand(0)?.as_number()?.real();
        let ac = op.operand(1)?.as_number()?.real();
        let content = op.operand(2)?.to_owned();
        //
        self.set_text_character_spacing(Operator::new(
            "Tc".to_string(),
            vec![PdfObject::Number(PdfNumber::Real(ac))],
        ))?;

        self.set_text_word_spacing(Operator::new(
            "Tw".to_string(),
            vec![PdfObject::Number(PdfNumber::Real(aw))],
        ))?;

        self.show_text(Operator::new("Tj".to_string(), vec![content]), device)
    }

    fn show_text_array(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        // TODO
        let params = op.operand(0)?.as_array()?;
        let wmd = self
            .state
            .font
            .as_ref()
            .ok_or(PdfError::Interpreter(
                "Show  text Array Font is None".to_string(),
            ))?
            .writting_mode();
        for operand in params.iter() {
            match operand {
                PdfObject::LiteralString(s) => {
                    let codes = s.bytes();
                    self.do_show_text(codes, device)?;
                }
                PdfObject::HexString(s) => {
                    let codes = s.raw_bytes()?;
                    self.do_show_text(codes.as_slice(), device)?;
                }
                PdfObject::Number(v) => {
                    let tj = v.real() * 0.001 * self.state.font_size * self.state.text_horz_scale();
                    match wmd {
                        WritingMode::Horizontal => {
                            let rm = Matrix::new_translation_matrix(-tj, 0.0);
                            let text_matrix = rm.transform(&self.state.text_matrix);
                            self.state.text_matrix = text_matrix;
                        }
                        WritingMode::Vertical => {
                            let rm = Matrix::new_translation_matrix(0.0, -tj);
                            let text_matrix = rm.transform(&self.state.text_matrix);
                            self.state.text_matrix = text_matrix;
                        }
                    }
                }
                _ => {
                    return Err(PdfError::Interpreter(format!(
                        "TJ impossiable:{:?}",
                        operand
                    )));
                }
            }
        }
        Ok(())
    }

    // g
    fn set_gray_fill(&mut self, op: Operator) -> Result<()> {
        let level = op
            .operand(0)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!(
                    "g operatand need to be a number got:{:?}",
                    op.operand(0)
                ))
            })?
            .real();
        self.state.fill_color_space = ColorSpace::DeviceGray(DeviceGray::new());
        self.state.fill_color_value = ColorValue::new(vec![level]);
        Ok(())
    }

    // G
    fn set_gray_stroke(&mut self, op: Operator) -> Result<()> {
        let level = op
            .operand(0)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!(
                    "G operator need a number level got:{:?}",
                    op.operand(0)
                ))
            })?
            .real();
        self.state.stroke_color_space = ColorSpace::DeviceGray(DeviceGray::new());
        self.state.stroke_color_value = ColorValue::new(vec![level]);
        Ok(())
    }

    // rg
    fn set_rgb_fill(&mut self, op: Operator) -> Result<()> {
        let r = op
            .operand(0)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!(
                    "rg Operator need a number got :{:?}",
                    op.operand(0)
                ))
            })?
            .real();
        let g = op
            .operand(1)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!(
                    "rg Operator need a number got :{:?}",
                    op.operand(1)
                ))
            })?
            .real();
        let b = op
            .operand(2)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!(
                    "rg Operator need a number got :{:?}",
                    op.operand(2)
                ))
            })?
            .real();
        self.state.fill_color_space = ColorSpace::DeviceRgb(DeviceRgb::new());
        self.state.fill_color_value = ColorValue::new(vec![r, g, b]);
        Ok(())
    }
    // RG
    fn set_rgb_stroke(&mut self, op: Operator) -> Result<()> {
        let r = op
            .operand(0)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!(
                    "RG Operator need a number got :{:?}",
                    op.operand(0)
                ))
            })?
            .real();
        let g = op
            .operand(1)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!(
                    "RG Operator need a number got :{:?}",
                    op.operand(1)
                ))
            })?
            .real();
        let b = op
            .operand(2)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!(
                    "RG Operator need a number got :{:?}",
                    op.operand(2)
                ))
            })?
            .real();
        self.state.stroke_color_space = ColorSpace::DeviceRgb(DeviceRgb::new());
        self.state.stroke_color_value = ColorValue::new(vec![r, g, b]);
        Ok(())
    }

    // k
    fn set_cmyk_fill(&mut self, op: Operator) -> Result<()> {
        let c = op
            .operand(0)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!("K Operator need a number got :{:?}", op.operand(0)))
            })?
            .real();
        let m = op
            .operand(1)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!("K Operator need a number got :{:?}", op.operand(0)))
            })?
            .real();
        let y = op
            .operand(2)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!("K Operator need a number got :{:?}", op.operand(0)))
            })?
            .real();
        let k = op
            .operand(3)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!("K Operator need a number got :{:?}", op.operand(0)))
            })?
            .real();
        self.state.fill_color_space = ColorSpace::DeviceCmyk(DeviceCmyk::new());
        self.state.fill_color_value = ColorValue::new(vec![c, m, y, k]);

        Ok(())
    }

    // K
    fn set_cmyk_stroke(&mut self, op: Operator) -> Result<()> {
        let c = op
            .operand(0)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!("k Operator need a number got :{:?}", op.operand(0)))
            })?
            .real();
        let m = op
            .operand(1)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!("k Operator need a number got :{:?}", op.operand(0)))
            })?
            .real();
        let y = op
            .operand(2)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!("k Operator need a number got :{:?}", op.operand(0)))
            })?
            .real();
        let k = op
            .operand(3)?
            .as_number()
            .map_err(|_| {
                PdfError::Interpreter(format!("k Operator need a number got :{:?}", op.operand(0)))
            })?
            .real();
        self.state.stroke_color_space = ColorSpace::DeviceCmyk(DeviceCmyk::new());
        self.state.stroke_color_value = ColorValue::new(vec![c, m, y, k]);
        Ok(())
    }

    // cs
    fn set_color_space_fill(&mut self, op: Operator) -> Result<()> {
        let operand = op.operand(0)?;
        let cname = operand.as_name()?;
        let color_space = match self.current_resource()?.lookup_color(cname.name()) {
            Some(cr) => parse_colorspace(cr, self.xref)?,
            None => parse_colorspace(operand, self.xref)?,
        };
        self.state.fill_color_value = color_space.default_value();
        self.state.fill_color_space = color_space;
        Ok(())
    }

    // CS
    fn set_color_space_stroke(&mut self, op: Operator) -> Result<()> {
        let operand = op.operand(0)?;
        let cname = operand.as_name()?;
        let color_space = match self.current_resource()?.lookup_color(cname.name()) {
            Some(cr) => parse_colorspace(cr, self.xref)?,
            None => parse_colorspace(operand, self.xref)?,
        };
        self.state.stroke_color_value = color_space.default_value();
        self.state.stroke_color_space = color_space;
        Ok(())
    }
    // sc
    fn set_color_fill(&mut self, op: Operator) -> Result<()> {
        match &self.state.fill_color_space {
            ColorSpace::DeviceGray(_) | ColorSpace::CalGray(_) | ColorSpace::Indexed(_) => {
                let v = op.operand(0).map_err(|_| {
                    PdfError::Interpreter(
                        "set fill color value DEviceGray CalGray Indexed colorspace need 1 param"
                            .to_string(),
                    )
                })?;
                let v = v.as_number().map_err(|_| {
                    PdfError::Interpreter(
                        "DeviceGray CalGray or Indexed colorspace need number as value".to_string(),
                    )
                })?;
                self.state.fill_color_value = ColorValue::new(vec![v.real()]);
            }
            ColorSpace::DeviceRgb(_) | ColorSpace::CalRgb(_) | ColorSpace::Lab(_) => {
                let mut values = Vec::with_capacity(3);
                for i in 0..3 {
                    let v = op
                        .operand(i)
                        .map_err(|_| {
                            PdfError::Interpreter(format!(
                                "set fill color value {} param is None  ",
                                i
                            ))
                        })?
                        .as_number()
                        .map_err(|_| {
                            PdfError::Interpreter(
                                "set fill Color value operand need to be number".to_string(),
                            )
                        })?;
                    values.push(v.real());
                }
                self.state.fill_color_value = ColorValue::new(values);
            }
            ColorSpace::DeviceCmyk(_) => {
                let mut values = Vec::with_capacity(4);
                for i in 0..4 {
                    let v = op
                        .operand(i)
                        .map_err(|_| {
                            PdfError::Interpreter(format!(
                                "set fill color value {} param is None  ",
                                i
                            ))
                        })?
                        .as_number()
                        .map_err(|_| {
                            PdfError::Interpreter(
                                "set fill Color value operand need to be number".to_string(),
                            )
                        })?;
                    values.push(v.real());
                }
                self.state.fill_color_value = ColorValue::new(values);
            }
            ColorSpace::DeviceN(_) | ColorSpace::IccBased(_) | ColorSpace::Separation(_) => {
                let n = op.num_operands();
                let mut values = Vec::with_capacity(n);
                for i in 0..n {
                    let v = op
                        .operand(i)
                        .unwrap()
                        .as_number()
                        .map_err(|_| {
                            PdfError::Interpreter(
                                "DeviceN, IccBased, Separation Colorspace value need to be number"
                                    .to_string(),
                            )
                        })?
                        .real();
                    values.push(v);
                }
                self.state.fill_color_value = ColorValue::new(values);
            }
            ColorSpace::Pattern(pattern) => {
                let pname = op.operand(0)?.as_name()?.name();
                let pobj = self
                    .current_resource()?
                    .lookup_pattern(pname)
                    .ok_or(PdfError::Interpreter("Pattern not found".to_string()))?;
                let pobj = self.xref.read_object(pobj)?;
                let pattern = Pattern::try_new(&pobj, self.xref);
                warn!("pattern:{:?}", pattern);
                //unimplemented!()
            }
        }
        Ok(())
    }

    // SC
    fn set_color_stroke(&mut self, op: Operator) -> Result<()> {
        Ok(())
    }

    // S
    fn stroke_path(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        if let Some(path) = self.current_path.as_ref() {
            device.stroke_path(path, &self.state)?;
            self.clear_current_path();
        } else {
            return Err(PdfError::Interpreter(
                "stroke path current_path is none".to_string(),
            ));
        }
        Ok(())
    }

    // s
    fn close_stroke_path(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        if let Some(path) = self.current_path.as_mut() {
            path.close_sub_path()?;
            device.stroke_path(path, &self.state)?;
            self.current_path = None;
            self.current_point = None;
        } else {
            return Err(PdfError::Interpreter(
                "stroke path current_path is none".to_string(),
            ));
        }
        Ok(())
    }

    // F
    fn fill_path_with_even_odd(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        if let Some(path) = self.current_path.as_ref() {
            device.fill_path(path, &self.state, FillRule::EvenOdd)?;
            self.clear_current_path();
        } else {
            return Err(PdfError::Interpreter(
                "stroke path current_path is none".to_string(),
            ));
        }
        Ok(())
    }

    // f
    fn fill_path_with_nonezero_winding(
        &mut self,
        op: Operator,
        device: &mut dyn Device,
    ) -> Result<()> {
        match self.state.fill_color_space {
            ColorSpace::Pattern(_) => {
                // TODO unsupport pattern
                self.clear_current_path();
                return Ok(());
            }
            _ => {
                if let Some(path) = self.current_path.as_mut() {
                    // TODO what is path close mean
                    //path.close_all().unwrap();
                    device.fill_path(path, &self.state, FillRule::Winding)?;
                } else {
                    return Err(PdfError::Interpreter(
                        "fill path current_path is none".to_string(),
                    ));
                }
                self.clear_current_path();
            }
        }
        Ok(())
    }

    fn clear_current_path(&mut self) {
        self.current_path = None;
        self.current_point = None;
    }

    fn fill_then_stroke_with_even_odd(
        &mut self,
        op: Operator,
        device: &mut dyn Device,
    ) -> Result<()> {
        if let Some(path) = self.current_path.as_mut() {
            device.fill_and_stroke_path(path, &self.state, FillRule::EvenOdd)?;
            self.clear_current_path();
        } else {
            return Err(PdfError::Interpreter(
                "fill_then storke path current_path is none".to_string(),
            ));
        }
        self.clear_current_path();
        Ok(())
    }

    fn fill_then_stroke_with_nonezer_winding(
        &mut self,
        op: Operator,
        device: &mut dyn Device,
    ) -> Result<()> {
        if let Some(path) = self.current_path.as_mut() {
            // TODO what is path close mean
            device.fill_and_stroke_path(path, &self.state, FillRule::Winding)?;
            self.clear_current_path();
        } else {
            return Err(PdfError::Interpreter(
                "fill_then storke path current_path is none".to_string(),
            ));
        }
        self.clear_current_path();
        Ok(())
    }
    fn close_fill_then_stroke_with_nonezer_winding(
        &mut self,
        op: Operator,
        device: &mut dyn Device,
    ) -> Result<()> {
        unimplemented!()
    }

    fn close_fill_then_stroke_with_even_odd(
        &mut self,
        op: Operator,
        device: &mut dyn Device,
    ) -> Result<()> {
        unimplemented!()
    }
    pub fn end_image(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        let img_info = op.operand(0)?.as_dict()?;
        let data = op.operand(1)?.as_literal()?;
        let image = PdfImage::try_new_inline(img_info, data.bytes(), self.xref)?;
        device.draw_image(image, &self.state)?;
        Ok(())
    }

    fn invoke_operator(&mut self, op: Operator, device: &mut dyn Device) -> Result<()> {
        let op_name = op.name();
        match op_name {
            // default
            "q" => self.push_graph_state(),
            "Q" => self.pop_graph_state(),
            "cm" => self.modify_current_transform_matrix(op, device),
            "Do" => self.do_operation(op, device),
            "BMC" => self.begin_marked_content(op),
            "BDC" => self.begin_marked_content_dictionary(op),
            "EMC" => self.end_marked_content(op),
            //// text
            "BT" => self.begin_text(device),
            "ET" => self.end_text(device),
            "Tc" => self.set_text_character_spacing(op),
            "Tw" => self.set_text_word_spacing(op),
            "Tz" => self.set_text_horizal_scaling(op),
            "TL" => self.set_text_leading(op),
            "Tf" => self.set_text_font(op, device),
            "Tr" => self.set_text_render_mode(op),
            "Ts" => self.set_text_rise_mode(op),
            "Td" => self.text_move_start_next_line(op),
            "TD" => self.text_move_start_next_line_with_leading(op),
            "Tm" => self.text_set_text_matrix(op),
            "T*" => self.text_set_move_next_line(),
            "Tj" => self.show_text(op, device),
            "'" => self.move_next_line_and_show_text(op, device),
            "\"" => self.move_text_line_and_show_text_with_leading(op, device),
            "TJ" => self.show_text_array(op, device),
            //// Create path
            "w" => self.set_line_width(op),
            "J" => self.set_line_cap(op),
            "j" => self.set_line_join(op),
            "M" => self.set_miter_limit(op),
            "d" => self.set_line_dash_pattern(op),
            "i" => self.set_flatness(op),
            "m" => self.move_to(op),
            "l" => self.line_to(op),
            "c" => self.curve_to(op),
            "v" => self.curve_first_point_duplicate(op),
            "re" => self.rectangle(op),
            "y" => self.curve_fourth_point_duplicate(op),
            "h" => self.close_sub_path(op),
            // PathPaint
            "S" => self.stroke_path(op, device),
            "s" => self.close_stroke_path(op, device),
            "F" | "f" => self.fill_path_with_nonezero_winding(op, device),
            "f*" => self.fill_path_with_even_odd(op, device),
            "B" => self.fill_then_stroke_with_nonezer_winding(op, device),
            "B*" => self.fill_then_stroke_with_even_odd(op, device),
            "b" => self.close_fill_then_stroke_with_nonezer_winding(op, device),
            "b*" => self.close_fill_then_stroke_with_even_odd(op, device),
            // clip path
            "n" => self.end_path(op),
            // ColorSpace
            "g" => self.set_gray_fill(op),
            "G" => self.set_gray_stroke(op),
            "rg" => self.set_rgb_fill(op),
            "RG" => self.set_rgb_stroke(op),
            "k" => self.set_cmyk_fill(op),
            "K" => self.set_cmyk_stroke(op),
            "cs" => self.set_color_space_fill(op),
            "CS" => self.set_color_space_stroke(op),
            "sc" | "scn" => self.set_color_fill(op),
            "SC" | "SCN" => self.set_color_stroke(op),
            "gs" => self.set_extend_graphic_state(op, device),
            //// Image
            "EI" => self.end_image(op, device),
            "ri" => self.set_render_intent(op),
            _ => Ok(()),
        }
    }
}
