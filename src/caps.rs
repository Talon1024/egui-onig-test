use std::{collections::VecDeque, cmp::Ordering::{self, *}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureInfo {
	pub group: Option<usize>,
	pub range: (usize, usize),
}

impl CaptureInfo {
	pub fn len(&self) -> usize {
		self.range.1 - self.range.0
	}
}

pub struct CaptureInfoFillIter {
	text_len: usize,
	items: Vec<EndPoint>,
	pos: usize,
	groups: VecDeque<usize>
}

impl CaptureInfoFillIter {
	pub fn new(citems: Vec<CaptureInfo>, text_len: usize) -> Self {
		let mut items = CaptureInfoFillIter::endpoint_list(&citems);
		items.sort_unstable();
		// println!("\n\n\n\n\n{:#?}", items);
		items.reverse();
		Self {
			text_len,
			items,
			pos: 0,
			groups: VecDeque::new(),
		}
	}

	fn endpoint_list(citems: &Vec<CaptureInfo>) -> Vec<EndPoint> {
		let mut items: Vec<EndPoint> = Vec::new();
		citems.iter().for_each(|item| {
			if let Some(group) = item.group {
				items.push(EndPoint {
					group,
					pos: item.range.0,
					etype: EndPointType::Start,
				});
				items.push(EndPoint {
					group,
					pos: item.range.1,
					etype: EndPointType::End,
				});
			}
		});
		items
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EndPoint {
	group: usize,
	pos: usize,
	etype: EndPointType,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EndPointType {
	Start,
	End,
}

impl PartialOrd for EndPoint {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		use EndPointType::*;
		let pos_order = self.pos.partial_cmp(&other.pos);
		if let Some(Equal) = pos_order {
			let order = self.group.partial_cmp(&other.group);
			// Group 0 is the boundary of a whole group, so some special
			// considerations are necessary.
			match (self.etype, other.etype) {
				(Start, Start) => order,
				(Start, End) => {
					match (self.group, other.group) {
						(0, 0) => Some(Greater),
						_ => Some(Less),
					}
				},
				(End, End) => order.map(Ordering::reverse),
				(End, Start) => {
					match (self.group, other.group) {
						(0, 0) => Some(Less),
						_ => Some(Greater),
					}
				},
			}
		} else {
			pos_order
		}
	}
}

impl Ord for EndPoint {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(other).unwrap()
	}
}

impl Iterator for CaptureInfoFillIter {
	type Item = CaptureInfo;
	fn next(&mut self) -> Option<Self::Item> {
		use EndPointType::*;
		loop {
		let next_endpoint = self.items.pop();
		match next_endpoint {
			Some(ep) => {
				let group = self.groups.back().copied();
				match ep.etype {
					Start => {
						self.groups.push_back(ep.group);
					},
					End => {
						if matches!(group, Some(g) if g != ep.group) {
							panic!("group {:?} != ep.group {}", group, ep.group);
						}
						self.groups.pop_back();
					},
				}
				if self.pos == ep.pos {
					continue;
				}
				let prev_pos = self.pos;
				self.pos = ep.pos;
				break Some(CaptureInfo {
					group,
					range: (prev_pos, self.pos),
				});
			},
			None => {
				if self.pos != self.text_len {
					let pos = self.pos;
					self.pos = self.text_len;
					break Some(CaptureInfo {
						group: None,
						range: (pos, self.text_len)
					});
				} else {
					break None;
				}
			},
		}
	}
	}
}

impl From<(usize, usize, usize)> for CaptureInfo {
	fn from(v: (usize, usize, usize)) -> Self {
		Self {
			group: Some(v.0),
			range: (v.1, v.2),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::error::Error;

	#[test]
	fn capture_info_fill_a() {
		// sa(u)er(k)raut
		// all sauerkraut wooff
		let text_len = 20;
		let caps = vec![
			CaptureInfo {group: Some(0), range: (4, 14)},
			CaptureInfo {group: Some(1), range: (6, 7)},
			CaptureInfo {group: Some(2), range: (9, 10)},
		];
		let expected = vec![
			CaptureInfo { group: None, range: (0, 4)},
			CaptureInfo { group: Some(0), range: (4, 6)},
			CaptureInfo { group: Some(1), range: (6, 7)},
			CaptureInfo { group: Some(0), range: (7, 9)},
			CaptureInfo { group: Some(2), range: (9, 10)},
			CaptureInfo { group: Some(0), range: (10, 14)},
			CaptureInfo { group: None, range: (14, 20)},
		];
		let filler = CaptureInfoFillIter::new(caps, text_len);
		let actual: Vec<CaptureInfo> = filler.collect();
		assert_eq!(expected.len(), actual.len());
		assert_eq!(expected, actual);
	}

	#[test]
	fn capture_info_fill_b() {
		// ((s)au)er(k)ra(u(t))
		// all sauerkraut wooff
		let text_len = 20;
		let caps = vec![
			CaptureInfo {group: Some(0), range: (4, 14)},
			CaptureInfo {group: Some(1), range: (4, 7)},
			CaptureInfo {group: Some(2), range: (4, 5)},
			CaptureInfo {group: Some(3), range: (9, 10)},
			CaptureInfo {group: Some(4), range: (12, 14)},
			CaptureInfo {group: Some(5), range: (13, 14)},
		];
		let expected = vec![
			CaptureInfo { group: None, range: (0, 4)},
			CaptureInfo { group: Some(2), range: (4, 5)},
			CaptureInfo { group: Some(1), range: (5, 7)},
			CaptureInfo { group: Some(0), range: (7, 9)},
			CaptureInfo { group: Some(3), range: (9, 10)},
			CaptureInfo { group: Some(0), range: (10, 12)},
			CaptureInfo { group: Some(4), range: (12, 13)},
			CaptureInfo { group: Some(5), range: (13, 14)},
			CaptureInfo { group: None, range: (14, 20)},
		];
		let filler = CaptureInfoFillIter::new(caps, text_len);
		let actual: Vec<CaptureInfo> = filler.collect();
		assert_eq!(expected.len(), actual.len());
		assert_eq!(expected, actual);
	}

	#[test]
	fn string_regex() -> Result<(), Box<dyn Error>> {
		use onig::Regex;
		let regex = Regex::new("(\\w+)\\s")?;
		let test_text = "Three words panic";
		let text_len = test_text.len();
		let test_captures = regex.captures_iter(test_text).flat_map(|found| {
			found.iter_pos().enumerate().filter_map(|(group_index, group)| {
				group.map(|(start, end)| CaptureInfo::from((group_index, start, end)))
			}).collect::<Vec<CaptureInfo>>()
		}).collect::<Vec<CaptureInfo>>();
		println!("test_captures before:\n{:?}\n\n\n", test_captures);
		let test_captures: Vec<CaptureInfo> = CaptureInfoFillIter::new(
			test_captures, text_len).collect();
		println!("test_captures after:\n{:?}\n\n\n", test_captures);
		Ok(())
	}

	#[test]
	fn endpoint_order() -> Result<(), Box<dyn Error>> {
		// ((s)au)er(k)ra(u(t))
		// all sauerkraut wooff
		use EndPointType::*;
		let mut endpoints = vec![
			EndPoint {group: 0, pos: 4, etype: Start},
			EndPoint {group: 0, pos: 14, etype: End},
			EndPoint {group: 1, pos: 4, etype: Start},
			EndPoint {group: 1, pos: 7, etype: End},
			EndPoint {group: 2, pos: 4, etype: Start},
			EndPoint {group: 2, pos: 5, etype: End},
			EndPoint {group: 3, pos: 9, etype: Start},
			EndPoint {group: 3, pos: 10, etype: End},
			EndPoint {group: 4, pos: 12, etype: Start},
			EndPoint {group: 4, pos: 14, etype: End},
			EndPoint {group: 5, pos: 13, etype: Start},
			EndPoint {group: 5, pos: 14, etype: End},
		];
		let expected = vec![
			EndPoint {group: 0, pos: 4, etype: Start},
			EndPoint {group: 1, pos: 4, etype: Start},
			EndPoint {group: 2, pos: 4, etype: Start},
			EndPoint {group: 2, pos: 5, etype: End},
			EndPoint {group: 1, pos: 7, etype: End},
			EndPoint {group: 3, pos: 9, etype: Start},
			EndPoint {group: 3, pos: 10, etype: End},
			EndPoint {group: 4, pos: 12, etype: Start},
			EndPoint {group: 5, pos: 13, etype: Start},
			EndPoint {group: 5, pos: 14, etype: End},
			EndPoint {group: 4, pos: 14, etype: End},
			EndPoint {group: 0, pos: 14, etype: End},
		];
		endpoints.sort_unstable();
		assert_eq!(expected, endpoints);
		Ok(())
	}

	#[test]
	fn endpoint_order_at_same_pos() -> Result<(), Box<dyn Error>> {
		use EndPointType::*;
		let mut endpoints = vec![
			EndPoint {group: 0, pos: 4, etype: Start},
			EndPoint {group: 0, pos: 14, etype: End},
			EndPoint {group: 5, pos: 5, etype: Start},
			EndPoint {group: 5, pos: 5, etype: End},
			EndPoint {group: 1, pos: 5, etype: Start},
			EndPoint {group: 1, pos: 5, etype: End},
			EndPoint {group: 3, pos: 5, etype: Start},
			EndPoint {group: 3, pos: 5, etype: End},
			EndPoint {group: 4, pos: 5, etype: Start},
			EndPoint {group: 4, pos: 5, etype: End},
			EndPoint {group: 2, pos: 5, etype: Start},
			EndPoint {group: 2, pos: 5, etype: End},
		];
		let expected = vec![
			EndPoint {group: 0, pos: 4, etype: Start},
			EndPoint {group: 1, pos: 5, etype: Start},
			EndPoint {group: 2, pos: 5, etype: Start},
			EndPoint {group: 3, pos: 5, etype: Start},
			EndPoint {group: 4, pos: 5, etype: Start},
			EndPoint {group: 5, pos: 5, etype: Start},
			EndPoint {group: 5, pos: 5, etype: End},
			EndPoint {group: 4, pos: 5, etype: End},
			EndPoint {group: 3, pos: 5, etype: End},
			EndPoint {group: 2, pos: 5, etype: End},
			EndPoint {group: 1, pos: 5, etype: End},
			EndPoint {group: 0, pos: 14, etype: End},
		];
		endpoints.sort_unstable();
		assert_eq!(expected, endpoints);
		Ok(())
	}

	#[test]
	fn endpoint_order_2() -> Result<(), Box<dyn Error>> {
		use EndPointType::*;
		let mut endpoints = vec![
			EndPoint {group: 0, pos: 0, etype: Start},
			EndPoint {group: 0, pos: 6, etype: End},
			EndPoint {group: 1, pos: 0, etype: Start},
			EndPoint {group: 1, pos: 5, etype: End},
			EndPoint {group: 0, pos: 6, etype: Start},
			EndPoint {group: 0, pos: 12, etype: End},
			EndPoint {group: 1, pos: 6, etype: Start},
			EndPoint {group: 1, pos: 11, etype: End},
		];
		let expected = vec![
			EndPoint {group: 0, pos: 0, etype: Start},
			EndPoint {group: 1, pos: 0, etype: Start},
			EndPoint {group: 1, pos: 5, etype: End},
			EndPoint {group: 0, pos: 6, etype: End},
			EndPoint {group: 0, pos: 6, etype: Start},
			EndPoint {group: 1, pos: 6, etype: Start},
			EndPoint {group: 1, pos: 11, etype: End},
			EndPoint {group: 0, pos: 12, etype: End},
		];
		endpoints.sort_unstable();
		assert_eq!(expected, endpoints);
		Ok(())
	}
}
