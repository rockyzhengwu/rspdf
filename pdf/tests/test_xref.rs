use pdf::document;

#[test]
fn test_xref_read() {
    let doc = document::Document::new_from_file("./tests/resources/hello_world.pdf", None).unwrap();
    assert_eq!(doc.objects_num(), 7);
}

#[test]
fn text_xref_multi_section() {
    let doc = document::Document::new_from_file("./tests/resources/empty_xref.pdf", None).unwrap();
    println!("{:?}", doc.objects_num());
    assert_eq!(doc.objects_num(), 7)
}

#[test]
fn test_xref_stream() {
    let doc = document::Document::new_from_file("./tests/resources/xref_stream.pdf", None).unwrap();
    assert_eq!(doc.objects_num(), 22);
}
