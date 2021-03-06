use rivi_loader::DebugOption;

fn main() {
    let a = vec![1.0f32; 64];
    let input = &vec![vec![a]];
    let mut output = vec![0.0f32; 1];

    let vk = rivi_loader::new(DebugOption::None).unwrap();

    let mut cursor = std::io::Cursor::new(&include_bytes!("./reduce/reduce.spv")[..]);
    let shader = vk.load_shader(&mut cursor, Some(vec![vec![2]])).unwrap();

    vk.compute(input, &mut output, &shader).unwrap();

    println!("Result: {:?}", output);
}
