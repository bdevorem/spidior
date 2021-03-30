//! This module is for constructing and executing
//! Nondeterministic Finite Automata

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    hash::Hash,
};

type Atom = char;

pub mod matcher;
pub mod replacer;

#[derive(Debug, Clone)]
enum TransitionType {
    Epsilon,
    Alpha(Atom),
    Range(String),
    NegativeRange(String),
    QuerySetRange(String),
    Open(u32),
    Close(u32),
}

#[derive(Debug, Clone)]
struct Transition {
    kind: TransitionType,
    dest: NodePointer,
}

impl Transition {
    fn new(kind: TransitionType, dest: NodePointer) -> Self {
        Self { kind, dest }
    }
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct NodePointer {
    id: usize,
}

impl NodePointer {
    fn new(id: usize) -> Self {
        Self { id }
    }
}
#[derive(Debug, Clone)]
pub struct Node {
    transitions: Vec<Transition>,
}
impl Node {
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Nfa {
    nodes: Vec<Node>,
}

impl Nfa {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    pub fn get(&self, i: &NodePointer) -> Option<&Node> {
        self.nodes.get(i.id)
    }

    pub fn new_node(&mut self) -> NodePointer {
        self.add_node(Node::new())
    }

    pub fn add_node(&mut self, node: Node) -> NodePointer {
        self.nodes.push(node);
        NodePointer::new(self.nodes.len() - 1)
    }

    pub fn add_transition_range(
        &mut self,
        from: &NodePointer,
        to: &NodePointer,
        s: String,
    ) -> Result<(), Box<dyn Error>> {
        self.add_transition(from, Transition::new(TransitionType::Range(s), *to))
    }

    pub fn add_transition_queryset(
        &mut self,
        from: &NodePointer,
        to: &NodePointer,
        s: String,
    ) -> Result<(), Box<dyn Error>> {
        self.add_transition(from, Transition::new(TransitionType::QuerySetRange(s), *to))
    }

    pub fn add_transition_negativerange(
        &mut self,
        from: &NodePointer,
        to: &NodePointer,
        s: String,
    ) -> Result<(), Box<dyn Error>> {
        self.add_transition(from, Transition::new(TransitionType::NegativeRange(s), *to))
    }

    pub fn add_transition_alpha(
        &mut self,
        from: &NodePointer,
        to: &NodePointer,
        on: Atom,
    ) -> Result<(), Box<dyn Error>> {
        self.add_transition(from, Transition::new(TransitionType::Alpha(on), *to))
    }

    pub fn add_transition_epsilon(
        &mut self,
        from: &NodePointer,
        to: &NodePointer,
    ) -> Result<(), Box<dyn Error>> {
        self.add_transition(from, Transition::new(TransitionType::Epsilon, *to))
    }

    pub fn add_group(
        &mut self,
        start_from: &NodePointer,
        start_to: &NodePointer,
        end_from: &NodePointer,
        end_to: &NodePointer,
        num: u32,
    ) -> Result<(), Box<dyn Error>> {
        self.add_transition(
            start_from,
            Transition::new(TransitionType::Open(num), *start_to),
        )?;
        self.add_transition(
            end_from,
            Transition::new(TransitionType::Close(num), *end_to),
        )
    }

    fn add_transition(&mut self, from: &NodePointer, to: Transition) -> Result<(), Box<dyn Error>> {
        let node = self.nodes.get_mut(from.id).ok_or("Invalid source!")?;
        node.transitions.push(to);
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Context {
    nodes: HashSet<NodePointer>,
}

impl Context {
    pub fn new(nodes: HashSet<NodePointer>) -> Self {
        Self { nodes }
    }

    pub fn contains(&self, i: &NodePointer) -> bool {
        return self.nodes.contains(i);
    }

    pub fn step(&self, nfa: &Nfa, input: Atom) -> Self {
        let mut nodes = HashSet::new();
        for nodeptr in &self.nodes {
            if let Some(node) = nfa.get(nodeptr) {
                for t in &node.transitions {
                    match &t.kind {
                        TransitionType::Alpha(c) if *c == input => {
                            nodes.insert(t.dest);
                        }
                        TransitionType::Range(s) if s.contains(input) => {
                            nodes.insert(t.dest);
                        }
                        TransitionType::NegativeRange(s) if !s.contains(input) => {
                            nodes.insert(t.dest);
                        }
                        _ => {}
                    }
                }
            }
        }
        Self::add_epsilons(nodes, nfa)
    }

    pub fn add_epsilons(nodes: HashSet<NodePointer>, nfa: &Nfa) -> Self {
        let mut nodes = nodes;
        loop {
            let prev = nodes.clone();
            let size = nodes.len();
            for nodeptr in &prev {
                if let Some(node) = nfa.get(nodeptr) {
                    for t in &node.transitions {
                        if let TransitionType::Epsilon = t.kind {
                            nodes.insert(t.dest);
                        }
                    }
                }
            }
            if size == nodes.len() {
                return Self::new(nodes);
            }
        }
    }
}

#[derive(Debug)]
pub struct NfaModel {
    nfa: Nfa,
    start: NodePointer,
    end: NodePointer,
}

impl NfaModel {
    pub fn to_dfa(&self) -> Result<Self, Box<dyn Error>> {
        let mut dfa = Nfa::new(Vec::new());
        let mut map = HashMap::new();
        let mut stack = Vec::new();
        let start = dfa.new_node();
        let end = dfa.new_node();
        let mut dfa_model = Self::new(dfa, start, end);
        let ctx = Context::add_epsilons(vec![start].into_iter().collect(), &dfa_model.nfa);
        let x: Vec<NodePointer> = ctx.nodes.into_iter().collect();
        map.insert(x.clone(), dfa_model.start);
        stack.push(x);
        while !stack.is_empty() {
            let x = stack.pop().ok_or("sad")?;
            let my_p = *map.get(&x).ok_or("how?")?;

            for old in &x {
                for new in &self.nfa.get(&old).ok_or("ahh")?.transitions {
                    if let TransitionType::Epsilon = new.kind {
                    } else {
                        let ctx = Context::add_epsilons(
                            vec![new.dest].into_iter().collect(),
                            &dfa_model.nfa,
                        );
                        println!("{:?}", ctx);
                        let super_state: Vec<NodePointer> = ctx.nodes.into_iter().collect();
                        let d = if let Some(new_p) = map.get(&super_state) {
                            *new_p
                        } else {
                            let y = dfa_model.nfa.new_node();
                            map.insert(super_state.clone(), y);
                            stack.push(super_state);
                            y
                        };
                        let t = Transition::new(new.kind.clone(), d);

                        dfa_model.nfa.add_transition(&my_p, t)?;
                    }
                }
            }
        }
        Ok(dfa_model)
    }
}

impl NfaModel {
    pub fn new(nfa: Nfa, start: NodePointer, end: NodePointer) -> Self {
        Self { nfa, start, end }
    }
}

#[test]
fn test_nfa_insert() -> Result<(), Box<dyn Error>> {
    let mut nfa = Nfa::new(Vec::new());
    nfa.add_node(Node::new());
    nfa.add_node(Node::new());
    nfa.add_node(Node::new());
    assert_eq!(nfa.nodes.len(), 3);
    Ok(())
}

#[test]
fn test_nfa_alpha_transition() -> Result<(), Box<dyn Error>> {
    let mut nfa = Nfa::new(Vec::new());
    let a = nfa.add_node(Node::new());
    let b = nfa.add_node(Node::new());
    nfa.add_transition_alpha(&a, &b, 'a')?;
    let ctx = Context::new(vec![a].into_iter().collect());
    let ctx2 = ctx.step(&nfa, 'b');
    assert_eq!(ctx2.nodes.len(), 0);
    let ctx2 = ctx.step(&nfa, 'a');
    assert_eq!(ctx2.nodes.len(), 1);
    assert!(ctx2.nodes.contains(&b));
    Ok(())
}

#[test]
fn test_nfa_epsilon_transition() -> Result<(), Box<dyn Error>> {
    let mut nfa = Nfa::new(Vec::new());
    let a = nfa.new_node();
    let b = nfa.new_node();
    let c = nfa.new_node();
    nfa.add_transition_alpha(&a, &b, 'a')?;
    nfa.add_transition_epsilon(&b, &c)?;
    let ctx = Context::new(vec![a].into_iter().collect());
    let ctx2 = ctx.step(&nfa, 'b');
    assert_eq!(ctx2.nodes.len(), 0);
    let ctx2 = ctx.step(&nfa, 'a');
    assert_eq!(ctx2.nodes.len(), 2);
    assert!(ctx2.nodes.contains(&b));
    assert!(ctx2.nodes.contains(&c));
    Ok(())
}

#[test]
fn test_nfa_to_dfa() -> Result<(), Box<dyn Error>> {
    let mut nfa = Nfa::new(Vec::new());
    let a = nfa.new_node();
    let b = nfa.new_node();
    let c = nfa.new_node();
    nfa.add_transition_alpha(&a, &b, 'a')?;
    nfa.add_transition_epsilon(&b, &c)?;
    let a = NfaModel::new(nfa, a, c);
    println!("{:?}", a.to_dfa());
    Ok(())
}
