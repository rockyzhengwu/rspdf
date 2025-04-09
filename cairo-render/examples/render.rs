use cairo_render::cairo_device::CairoDevice;
use pdf::document::Document;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Uage target/render [path]")
    }
    let path = &args[1];
    let password = None;

    let doc = Document::new_from_file(path, password).unwrap();
    let mut device = CairoDevice::new(300.0, 300.0);

    for page_num in 0..doc.total_page().unwrap() {
        println!("start page: {:?}", page_num);
        let page = doc.get_page(&page_num).unwrap();
        page.display(page_num, &mut device).unwrap();
    }
}
