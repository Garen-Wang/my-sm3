use my_sm3::Sm3;

#[test]
fn sm3_integration_test1() {
    let message_str = "abc";
    let message = message_str.as_bytes();

    let res = Sm3::digest(message);
    println!("{:x?}", res);
}

#[test]
fn sm3_integration_test2() {
    let message_str = "abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd";
    let message = message_str.as_bytes();

    let res = Sm3::digest(message);
    println!("{:x?}", res);
}
