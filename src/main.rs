
fn main() {
    if os::args().len() == 1 {
        fail ~"Filename required";
    }
    let res = io::file_reader(&path::Path(os::args()[1]));
    if result::is_err(&res) {
        fail result::get_err(&res);
    }
    let fread = result::unwrap(res);
    class_file::ClassFile(fread as io::ReaderUtil);
}
