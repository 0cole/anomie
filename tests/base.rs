use anomie::fuzz_binary;

#[test]
fn it_works() {
    fuzz_binary("", 12312);
    assert_eq!(2 + 2, 4);
}
