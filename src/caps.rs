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
		items.sort_unstable();
		// println!("{:?}", items);
		items.reverse();
		Self {
			text_len,
			items,
			pos: 0,
			groups: VecDeque::new(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EndPoint {
	group: usize,
	pos: usize,
	etype: EndPointType,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EndPointType {
	Start,
	End,
}

impl PartialOrd for EndPoint {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		let pos_order = self.pos.partial_cmp(&other.pos);
		if let Some(Equal) = pos_order {
			let mut order = self.group.partial_cmp(&other.group);
			if self.etype == EndPointType::End {
				order = order.map(Ordering::reverse);
			}
			order
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
						let group = self.groups.back().copied();
						if matches!(group, Some(g) if g != ep.group) {
							panic!("group {:?} != ep.group {}", group, ep.group);
						}
						self.groups.pop_back();
					},
				}
				let prev_pos = self.pos;
				self.pos = ep.pos;
				if prev_pos == ep.pos {
					continue;
				}
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
		// ((s)au)er(k)rau(t)
		// all sauerkraut wooff
		let text_len = 20;
		let caps = vec![
			CaptureInfo {group: Some(0), range: (4, 14)},
			CaptureInfo {group: Some(1), range: (4, 7)},
			CaptureInfo {group: Some(2), range: (4, 5)},
			CaptureInfo {group: Some(3), range: (9, 10)},
			CaptureInfo {group: Some(4), range: (13, 14)},
		];
		let expected = vec![
			CaptureInfo { group: None, range: (0, 4)},
			CaptureInfo { group: Some(2), range: (4, 5)},
			CaptureInfo { group: Some(1), range: (5, 7)},
			CaptureInfo { group: Some(0), range: (7, 9)},
			CaptureInfo { group: Some(3), range: (9, 10)},
			CaptureInfo { group: Some(0), range: (10, 13)},
			CaptureInfo { group: Some(4), range: (13, 14)},
			CaptureInfo { group: None, range: (14, 20)},
		];
		let filler = CaptureInfoFillIter::new(caps, text_len);
		let actual: Vec<CaptureInfo> = filler.collect();
		assert_eq!(expected.len(), actual.len());
		assert_eq!(expected, actual);
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
