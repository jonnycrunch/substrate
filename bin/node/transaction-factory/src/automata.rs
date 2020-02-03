// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Implementation of a kind of automata, used to read benchmark tests from files.

use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Edge {
	/// Sender name for this transaction.
	sender: String,
	/// Target state.
	target: u32,
	/// Module of the transaction to execute.
	tx_module: String,
	/// Name of the transaction to executed.
	tx_name: String,
	/// Name of transaction parameters that will be used.
	tx_params: Vec<String>,
	/// Maximum number of times we can move through it.
	repeat: u32,
	/// Number of times we passed through it.
	used: u32, 
}

#[derive(Debug)]
pub struct Node {
	outputs: Vec<Edge>,
}

#[derive(Debug)]
pub struct Automaton {
	nodes: HashMap<u32, Node>,
	current_node: u32,
	looping: Option<u32>,
	exit_state: u32,
	difficulty: u32,
}

impl Automaton {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new(),
			current_node: 0,
			looping: None,
			exit_state: 0,
			difficulty: 0,
		}
	}

	pub fn new_from_file(file_name_path: &str, difficulty: u32) -> Self {
		let contents = fs::read_to_string(file_name_path)
			.expect("something went wrong reading the bench file");
		let mut automaton = Automaton::new();
		automaton.difficulty = difficulty;
		
		for line in contents.lines() {
			let line: Vec<&str> = line.split_whitespace().collect();
			let source = line[0].parse().expect("source value can't be parsed");
			let target = line[1].parse().expect("target value can't be parsed");
			let tx_module = line[2].to_string();
			let sender = line[3].to_string();
			let tx_name = line[4].to_string();
			let tx_params = line.get(5).unwrap_or(&",")
				.split(",")
				.map(|s| s.to_string())
				.collect::<Vec<String>>();
			let repeat = line.get(6).unwrap_or(&"1").parse().expect("repeat value can't be parsed");

			let edge = Edge {
				target,
				sender,
				tx_module,
				tx_name,
				tx_params,
				repeat,
				used: 0,
			};

			if let Some(node) = automaton.nodes.get_mut(&source) {
				node.outputs.push(edge);
			} else {
				let new_node = Node { outputs: vec![edge] };
				automaton.nodes.insert(source, new_node);
			}
		}

		automaton
	}

	pub fn next_state(&mut self) -> Option<(String, String, String, Vec<String>, Option<u32>)> {
		if let Some(node) = self.nodes.get_mut(&self.current_node) {
			let mut max_out: Option<&mut Edge> = None;

			for edge in node.outputs.iter_mut() {
				if let Some(max_edge) = max_out.take() {
					if (*edge).repeat > max_edge.repeat && (*edge).repeat - (*edge).used > 0 {
						max_out = Some(edge);
					} else {
						max_out = Some(max_edge);
					}
				} else if (*edge).repeat - (*edge).used > 0 {
					max_out = Some(edge);
				}
			}

			if let Some(edge) = max_out {
				edge.used += 1;
				self.current_node = edge.target;

				Some((edge.sender.clone(), edge.tx_module.clone(), edge.tx_name.clone(), edge.tx_params.clone(), Some(self.difficulty)))
			} else {
				None
			}
		} else {
			panic!("automaton {} node is undefined, check your file!", self.current_node);
		}
	}

	pub fn clear_usage(&mut self) {
		for (_, node) in self.nodes.iter_mut() {
			for edge in node.outputs.iter_mut() {
				edge.used = 0;
			}
		}
		self.current_node = 0;
	}
}

