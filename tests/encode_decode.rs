use exp_golomb::*;
use rand::{Rng, SeedableRng};

#[test]
fn encode_decode() {
    let nums = [0, 1, 2, 3, 4, 5, 6, 7, 8];

    let mut buf = [0u8; 6];
    let mut writer = ExpGolombEncoder::new(&mut buf, 0).unwrap();

    for &num in &nums {
        writer.put_unsigned(num).unwrap();
    }
    writer.close();

    let mut reader = ExpGolombDecoder::new(&buf, 0).unwrap();
    for &num in &nums {
        assert_eq!(reader.next_unsigned(), Some(num));
    }
}

#[test]
fn encode_decode_random() {
    const SEED: u64 = 0;
    const NUM_VALS: usize = 1000;

    let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);
    let nums: Vec<_> = (0..NUM_VALS).map(|_| rng.gen::<u64>()).collect();

    let mut buf = vec![0u8; 3 * 8 * NUM_VALS];
    let mut writer = ExpGolombEncoder::new(&mut buf, 0).unwrap();

    for &num in &nums {
        writer.put_unsigned(num).unwrap();
    }
    writer.close();

    let mut reader = ExpGolombDecoder::new(&buf, 0).unwrap();
    for &num in &nums {
        assert_eq!(reader.next_unsigned(), Some(num));
    }
}
