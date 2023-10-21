// SPDX-License-Identifier: Apache-2.0

use primwrap::Primitive;

#[derive(Copy, Clone, Debug, Primitive)]
struct Int(u32);
#[derive(Copy, Clone, Debug, Primitive)]
struct Bool(bool);

fn main() {
	let wrapped = Int(5);
	assert_eq!(wrapped + 7, 12);
	assert_eq!(wrapped - 3, 2);
	assert_eq!(wrapped * 7, 35);
	assert_eq!(wrapped / 2, 2);
	assert!(wrapped > 0);
	assert!(10 > wrapped);

	let wrapped = Bool(true);
	assert_eq!(wrapped, true);
}