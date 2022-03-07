struct VariableId(u64);

enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Term { functor: String, args: Vec<Value>, annotations: Vec<Value> },
    List(Box<List>),
    Variable(VariableId),
    UnaryOp { op: UnaryOparator, value: Box<Value> },
    BinaryOp { op: BinaryOperator, left: Box<Value>, right: Box<Value> },
}

enum List {
    Empty,
    Element { head: Value, tail: Box<List> }
}

enum UnaryOparator {
    Neg,
}

enum BinaryOperator {
    Plus,
}
