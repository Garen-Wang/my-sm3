use my_sm3::Sm3;

fn main() {
    env_logger::init();

    // let message_str = "abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd";
    let message_str = "abc";
    let message = message_str.as_bytes();

    let res = Sm3::digest(message);
    println!("{:x?}", res);
}