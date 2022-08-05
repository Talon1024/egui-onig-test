use std::{collections::VecDeque, cmp::Ordering::{self, *}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureInfo {
	pub group: Option<usize>,
	pub range: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputCaptureInfo {
	pub mgroup: usize,
	pub group: Option<usize>,
	pub range: (usize, usize),
}

impl From<(usize, usize, usize, usize)> for InputCaptureInfo {
	fn from(v: (usize, usize, usize, usize)) -> Self {
		Self {
			mgroup: v.0,
			group: Some(v.1),
			range: (v.2, v.3),
		}
	}
}

pub struct CaptureInfoFillIter {
	text_len: usize,
	items: Vec<EndPoint>,
	pos: usize,
	groups: VecDeque<usize>
}

impl CaptureInfoFillIter {
	pub fn new(citems: Vec<InputCaptureInfo>, text_len: usize) -> Self {
		let mut items = CaptureInfoFillIter::endpoint_list(&citems);
		items.sort_unstable();
		if cfg!(test) {
			println!("\n\n\n\n\n{:?}", items);
		}
		items.reverse();
		Self {
			text_len,
			items,
			pos: 0,
			groups: VecDeque::new(),
		}
	}

	fn endpoint_list(citems: &Vec<InputCaptureInfo>) -> Vec<EndPoint> {
		let mut items: Vec<EndPoint> = Vec::new();
		citems.iter().for_each(|item| {
			if let Some(group) = item.group {
				items.push(EndPoint {
					mgroup: item.mgroup,
					group,
					pos: item.range.0,
					etype: EndPointType::Start,
				});
				items.push(EndPoint {
					mgroup: item.mgroup,
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
	mgroup: usize,
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
		let mg_order = self.mgroup.partial_cmp(&other.mgroup);
		if let Some(Equal) = mg_order {
		let pos_order = self.pos.partial_cmp(&other.pos);
		if let Some(Equal) = pos_order {
			let gorder = self.group.partial_cmp(&other.group);
			match (self.etype, other.etype) {
				(Start, _) => gorder,
				(End, _) => gorder.map(Ordering::reverse),
			}
		} else {
			pos_order
		}
	} else {
		mg_order
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

#[cfg(test)]
mod tests {
	use super::*;
	use std::error::Error;

	#[test]
	fn capture_info_fill_no_ends() {
		// sa(u)er(k)raut
		// all sauerkraut wooff
		let text_len = 20;
		let caps = vec![
			InputCaptureInfo {mgroup: 0, group: Some(0), range: (4, 14)},
			InputCaptureInfo {mgroup: 0, group: Some(1), range: (6, 7)},
			InputCaptureInfo {mgroup: 0, group: Some(2), range: (9, 10)},
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
	fn capture_info_fill_with_ends() {
		// ((s)au)er(k)ra(u(t))
		// all sauerkraut wooff
		let text_len = 20;
		let caps = vec![
			InputCaptureInfo {mgroup: 0, group: Some(0), range: (4, 14)},
			InputCaptureInfo {mgroup: 0, group: Some(1), range: (4, 7)},
			InputCaptureInfo {mgroup: 0, group: Some(2), range: (4, 5)},
			InputCaptureInfo {mgroup: 0, group: Some(3), range: (9, 10)},
			InputCaptureInfo {mgroup: 0, group: Some(4), range: (12, 14)},
			InputCaptureInfo {mgroup: 0, group: Some(5), range: (13, 14)},
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
		let test_captures = regex.captures_iter(test_text).enumerate()
		.flat_map(|(cap_group, found)| {
			found.iter_pos().enumerate().filter_map(|(group_index, group)| {
				group.map(|(start, end)| InputCaptureInfo::from(
					(cap_group, group_index, start, end)))
			}).collect::<Vec<InputCaptureInfo>>()
		}).collect::<Vec<InputCaptureInfo>>();
		println!("test_captures before:\n{:?}\n\n\n", test_captures);
		let test_captures: Vec<CaptureInfo> = CaptureInfoFillIter::new(
			test_captures, text_len).collect();
		println!("test_captures after:\n{:?}\n\n\n", test_captures);
		Ok(())
	}

	#[test]
	fn string_regex_whole() -> Result<(), Box<dyn Error>> {
		use onig::Regex;
		let regex = Regex::new("(\\w+)(\\s)")?;
		let test_text = "Three words panic";
		let text_len = test_text.len();
		let test_captures = regex.captures_iter(test_text).enumerate()
		.flat_map(|(cap_group, found)| {
			found.iter_pos().enumerate().filter_map(|(group_index, group)| {
				group.map(|(start, end)| InputCaptureInfo::from(
					(cap_group, group_index, start, end)))
			}).collect::<Vec<InputCaptureInfo>>()
		}).collect::<Vec<InputCaptureInfo>>();
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
			EndPoint {mgroup: 0, group: 0, pos: 4, etype: Start},
			EndPoint {mgroup: 0, group: 0, pos: 14, etype: End},
			EndPoint {mgroup: 0, group: 1, pos: 4, etype: Start},
			EndPoint {mgroup: 0, group: 1, pos: 7, etype: End},
			EndPoint {mgroup: 0, group: 2, pos: 4, etype: Start},
			EndPoint {mgroup: 0, group: 2, pos: 5, etype: End},
			EndPoint {mgroup: 0, group: 3, pos: 9, etype: Start},
			EndPoint {mgroup: 0, group: 3, pos: 10, etype: End},
			EndPoint {mgroup: 0, group: 4, pos: 12, etype: Start},
			EndPoint {mgroup: 0, group: 4, pos: 14, etype: End},
			EndPoint {mgroup: 0, group: 5, pos: 13, etype: Start},
			EndPoint {mgroup: 0, group: 5, pos: 14, etype: End},
		];
		let expected = vec![
			EndPoint {mgroup: 0, group: 0, pos: 4, etype: Start},
			EndPoint {mgroup: 0, group: 1, pos: 4, etype: Start},
			EndPoint {mgroup: 0, group: 2, pos: 4, etype: Start},
			EndPoint {mgroup: 0, group: 2, pos: 5, etype: End},
			EndPoint {mgroup: 0, group: 1, pos: 7, etype: End},
			EndPoint {mgroup: 0, group: 3, pos: 9, etype: Start},
			EndPoint {mgroup: 0, group: 3, pos: 10, etype: End},
			EndPoint {mgroup: 0, group: 4, pos: 12, etype: Start},
			EndPoint {mgroup: 0, group: 5, pos: 13, etype: Start},
			EndPoint {mgroup: 0, group: 5, pos: 14, etype: End},
			EndPoint {mgroup: 0, group: 4, pos: 14, etype: End},
			EndPoint {mgroup: 0, group: 0, pos: 14, etype: End},
		];
		endpoints.sort_unstable();
		assert_eq!(expected, endpoints);
		test_group_stack(&endpoints)
	}

	#[test]
	fn endpoint_order_at_same_pos() -> Result<(), Box<dyn Error>> {
		use EndPointType::*;
		let mut endpoints = vec![
			EndPoint {mgroup: 0, group: 0, pos: 5, etype: Start},
			EndPoint {mgroup: 0, group: 0, pos: 5, etype: End},
			EndPoint {mgroup: 0, group: 5, pos: 5, etype: Start},
			EndPoint {mgroup: 0, group: 5, pos: 5, etype: End},
			EndPoint {mgroup: 0, group: 1, pos: 5, etype: Start},
			EndPoint {mgroup: 0, group: 1, pos: 5, etype: End},
			EndPoint {mgroup: 0, group: 3, pos: 5, etype: Start},
			EndPoint {mgroup: 0, group: 3, pos: 5, etype: End},
			EndPoint {mgroup: 0, group: 4, pos: 5, etype: Start},
			EndPoint {mgroup: 0, group: 4, pos: 5, etype: End},
			EndPoint {mgroup: 0, group: 2, pos: 5, etype: Start},
			EndPoint {mgroup: 0, group: 2, pos: 5, etype: End},
		];
		endpoints.sort_unstable();
		test_group_stack(&endpoints)
	}

	#[test]
	fn endpoint_order_multiple_mgroups() -> Result<(), Box<dyn Error>> {
		use EndPointType::*;
		let mut endpoints = vec![
			EndPoint {mgroup: 0, group: 0, pos: 0, etype: Start},
			EndPoint {mgroup: 0, group: 0, pos: 6, etype: End},
			EndPoint {mgroup: 0, group: 1, pos: 0, etype: Start},
			EndPoint {mgroup: 0, group: 1, pos: 5, etype: End},
			EndPoint {mgroup: 1, group: 0, pos: 6, etype: Start},
			EndPoint {mgroup: 1, group: 0, pos: 12, etype: End},
			EndPoint {mgroup: 1, group: 1, pos: 6, etype: Start},
			EndPoint {mgroup: 1, group: 1, pos: 11, etype: End},
		];
		let expected = vec![
			EndPoint {mgroup: 0, group: 0, pos: 0, etype: Start},
			EndPoint {mgroup: 0, group: 1, pos: 0, etype: Start},
			EndPoint {mgroup: 0, group: 1, pos: 5, etype: End},
			EndPoint {mgroup: 0, group: 0, pos: 6, etype: End},
			EndPoint {mgroup: 1, group: 0, pos: 6, etype: Start},
			EndPoint {mgroup: 1, group: 1, pos: 6, etype: Start},
			EndPoint {mgroup: 1, group: 1, pos: 11, etype: End},
			EndPoint {mgroup: 1, group: 0, pos: 12, etype: End},
		];
		endpoints.sort_unstable();
		assert_eq!(expected, endpoints);
		test_group_stack(&endpoints)
	}

	fn test_group_stack(endpoints: &[EndPoint]) -> Result<(), Box<dyn Error>> {
		use EndPointType::*;
		let mut group_stack = VecDeque::new();
		endpoints.iter().map(|ep| {
			match ep.etype {
				Start => {
					group_stack.push_back(ep.group);
					Ok(())
				},
				End => {
					let group = group_stack.back().copied();
					if let Some(group) = group {
						if group != ep.group {
	return Err(Box::from(format!(
		"Wrong group! Expected {}, got {}",
		group, ep.group)));
						}
						group_stack.pop_back();
						Ok(())
					} else {
						Err(Box::from(format!("Empty group stack!")))
					}
				}
			}
		}).collect()
	}
}
