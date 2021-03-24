//! This module is for constructing and executing
//! Nondeterministic Finite Automata

use std::{collections::HashSet, error::Error, hash::Hash};

type Atom = char;

#[derive(Debug, Clone)]
enum NodeType {
    Accepting,
    Rejecting,
}

#[derive(Debug, Clone)]
enum TransitionType {
    Epsilon,
    Alpha(Atom),
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
    kind: NodeType,
}
impl Node {
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
            kind: NodeType::Rejecting,
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

    fn add_transition(&mut self, from: &NodePointer, to: Transition) -> Result<(), Box<dyn Error>> {
        let node = self.nodes.get_mut(from.id).ok_or("Invalid source!")?;
        node.transitions.push(to);
        Ok(())
    }
}
pub struct Context {
    nodes: HashSet<NodePointer>,
}

impl Context {
    fn new(nodes: HashSet<NodePointer>) -> Self {
        Self { nodes }
    }

    fn step(&self, nfa: Nfa, input: Atom) -> Self {
        let mut nodes = HashSet::new();
        for nodeptr in &self.nodes {
            if let Some(node) = nfa.get(nodeptr) {
                for t in &node.transitions {
                    if let TransitionType::Alpha(c) = t.kind {
                        if c == input {
                            nodes.insert(t.dest);
                        }
                    }
                }
            }
        }
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
    let ctx2 = ctx.step(nfa.clone(), 'b');
    assert_eq!(ctx2.nodes.len(), 0);
    let ctx2 = ctx.step(nfa, 'a');
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
    let ctx2 = ctx.step(nfa.clone(), 'b');
    assert_eq!(ctx2.nodes.len(), 0);
    let ctx2 = ctx.step(nfa, 'a');
    assert_eq!(ctx2.nodes.len(), 2);
    assert!(ctx2.nodes.contains(&b));
    assert!(ctx2.nodes.contains(&c));
    Ok(())
}
