// SPDX-License-Identifier: Apache-2.0

use primwrap::Primitive;

#[derive(Copy, Clone, Primitive)]
struct Int(u32);
#[derive(Copy, Clone, Primitive)]
struct Bool(bool);
#[derive(Copy, Clone, Primitive)]
struct Float(f32);

fn main() {
	let wrapped = Int(5);
	assert_eq!(wrapped + 7, 12);
	assert_eq!(wrapped - 3, 2);
	assert_eq!(wrapped * 7, 35);
	assert_eq!(wrapped / 2, 2);
	assert!(wrapped > 0);
	assert!(10 > wrapped);

	let wrapped = Int(0xFF00);
	assert_eq!(wrapped & 0xFF, 0);
	assert_eq!(wrapped | 0xFF, 0xFFFF);
	assert_eq!(wrapped ^ 0, 0xFF00);
	assert_eq!(wrapped << 4u8, 0xFF000);
	assert_eq!(wrapped >> 4u8, 0xFF0);
	assert_eq!(!wrapped, 0xFFFF00FF);

	let wrapped = Bool(true);
	assert_eq!(wrapped, true);
	assert_eq!(wrapped & false, false);
	assert_eq!(wrapped | false, true);
	assert_eq!(wrapped ^ true, false);
	assert_eq!(!wrapped, false);

	let wrapped = Float(5.0);
	assert_eq!(wrapped + 7.0, 12.0);
	assert_eq!(wrapped - 3.0, 2.0);
	assert_eq!(wrapped * 7.0, 35.0);
	assert_eq!(wrapped / 2.0, 2.5);
}