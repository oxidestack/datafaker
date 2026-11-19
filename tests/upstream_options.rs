//! `OptionsTest` (Datafaker 2.7.0, `providers/base/OptionsTest.java`). Java's boxed-number and `BigInteger` arrays
//! become Rust slices of the corresponding types; `option(Day.class)` becomes `option(&Day::VALUES)`, which draws
//! as `nextEnum` does.

use datafaker::Faker;

fn opt() -> Faker {
    Faker::new()
}

#[test]
fn test_option_with_array() {
    let options = ["A", "B", "C"];
    assert!(options.contains(&opt().options().option(&options).unwrap()));
}

#[test]
fn test_option_with_varargs_string() {
    assert!(["A", "B", "C"].contains(&opt().options().option(&["A", "B", "C"]).unwrap()));
}

#[test]
fn test_option_with_varargs() {
    let f = opt();
    let o = f.options();
    let ints = [1, 3, 4, 5];
    assert!(ints.contains(&o.option(&ints).unwrap()));
    let longs = [1i64, 3, 4, 5];
    assert!(longs.contains(&o.option(&longs).unwrap()));
    let shorts = [1i16, 3, 4];
    assert!(shorts.contains(&o.option(&shorts).unwrap()));
    let bytes = [11i8, 13, 14];
    assert!(bytes.contains(&o.option(&bytes).unwrap()));
    let doubles = [1.1f64, 13.0, 14.2];
    assert!(doubles.contains(&o.option(&doubles).unwrap()));
    let floats = [1.2f32, 13.0, 14.2];
    assert!(floats.contains(&o.option(&floats).unwrap()));
    let big = [1u128, 10, 0];
    assert!(big.contains(&o.option(&big).unwrap()));
    let bools = [true, false];
    assert!(bools.contains(&o.option(&bools).unwrap()));
}

#[test]
fn test_subset() {
    let f = opt();
    let o = f.options();
    let ints = [1, 3, 4, 5];
    let s = o.subset_of(1, &ints).unwrap();
    assert!(!s.contains(&2) && !s.contains(&6) && s.iter().any(|x| ints.contains(x)));
    let longs = [1i64, 3, 4, 5];
    let s = o.subset_of(1, &longs).unwrap();
    assert!(!s.contains(&2) && !s.contains(&6) && s.iter().any(|x| longs.contains(x)));
    assert_eq!(o.subset_of(longs.len() as i32, &longs).unwrap().len(), longs.len());
    assert_eq!(o.subset_of(longs.len() as i32 + 1, &longs).unwrap().len(), longs.len());
    let strs = ["1", "2", "3"];
    let s = o.subset(strs.len() as i32 + 1, &strs).unwrap();
    assert_eq!(s.len(), 3);
    assert!(s.iter().all(|x| strs.contains(&x.as_str())));
    let s = o.subset(1, &strs).unwrap();
    assert_eq!(s.len(), 1);
    assert!(strs.contains(&s[0].as_str()));
}

#[test]
fn test_subset_with_duplicate() {
    let f = opt();
    assert_eq!(f.options().subset_of(5, &[1, 1, 2, 2]).unwrap().len(), 2);
    assert_eq!(f.options().subset(i32::MAX, &["a", "s", "s", "a"]).unwrap().len(), 2);
}

#[test]
fn test_empty_subset() {
    let f = opt();
    assert!(f.options().subset_of(0, &[1, 2, 3]).unwrap().is_empty());
    assert_eq!(f.options().subset_of(-1, &[1, 2, 3]).unwrap_err().simple_class(), "IllegalArgumentException");
    assert!(f.options().subset(0, &["1", "2", "3"]).unwrap().is_empty());
    assert_eq!(f.options().subset(-1, &["1", "2", "3"]).unwrap_err().simple_class(), "IllegalArgumentException");
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

const DAYS: [Day; 7] = [Day::Monday, Day::Tuesday, Day::Wednesday, Day::Thursday, Day::Friday, Day::Saturday, Day::Sunday];

#[test]
fn test_option_with_enum() {
    assert!(DAYS.contains(&opt().options().option(&DAYS).unwrap()));
}

#[test]
fn test_next_array_element() {
    let f = opt();
    let array = [1, 2, 3, 5, 8, 13, 21];
    for _ in 1..10 {
        assert!(array.contains(&f.options().next_element(&array).unwrap()));
    }
}

#[test]
fn test_next_list_element() {
    let f = opt();
    let list = vec![1, 2, 3, 5, 8, 13, 21];
    for _ in 1..10 {
        assert!(list.contains(&f.options().next_element(&list).unwrap()));
    }
}
