// SPDX-License-Identifier: Apache-2.0

use primwrap::Primitive;

#[derive(Copy, Clone, Debug, Primitive)]
struct Wrapper(u32);

fn main() {
	let wrapped = Wrapper(5);
	assert_eq!(wrapped + 7, 12);
	assert_eq!(wrapped - 3, 2);
	assert_eq!(wrapped * 7, 35);
	assert_eq!(wrapped / 2, 2);
	assert!(wrapped > 0);
	assert!(10 > wrapped);
}