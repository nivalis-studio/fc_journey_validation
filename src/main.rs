use gps_trajectory_validation::validation;

fn main() {
    let t1 = vec![1.0, 1.0, 3.0, 3.0, 5.0, 5.0, 7.0, 7.0, 8.0, 8.0];
    let t2 = vec![1.0, 1.0, 5.0, 5.0, 8.0, 8.0];

    let res = validation(&t1, &t2).unwrap();
    println!("{:?}", res);
}
