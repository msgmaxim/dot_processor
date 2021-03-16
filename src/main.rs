
#[derive(Debug)]
struct DotFile {
    nodes: Vec<Node>,
    links: Vec<Link>
}

#[derive(Debug, Clone, PartialEq)]
struct Assignment {
    var: String,
    val: String,
}


type NodeId = String;
#[derive(Debug, Clone)]
struct Node {
    id: NodeId,
    label: String,
    assignments: Vec<Assignment>,
    is_root: bool,
}

#[derive(Debug)]
struct Link {
    from: NodeId,
    to: NodeId,
    label: String,
    color: String,
    fontcolor: String,
}

fn parse_node(line: &str) -> Node {

    // println!("{}", line);
    let idx = line.find(' ').unwrap();

    let (id, rest) = line.split_at(idx);

    let label = find_property(rest, "label").unwrap();

    let assignments = parse_label(&label);

    Node { id: id.to_owned(), label, is_root: false, assignments }
}

fn find_property(line: &str, name: &str) -> Option<String> {

    let pos = line.find(name)?;

    let (_, rest) = line.split_at(pos + name.len() + 2);

    let mut temp_rest = rest;

    let mut pos = rest.find("\"")?;

    // Make sure we ignore escaped characters
    while (temp_rest.find("\\\"").is_some()) && (pos == temp_rest.find("\\\"").unwrap() + 1) {

        let (_, rhs) = temp_rest.split_at(pos + 1);
        temp_rest = rhs;

        pos = temp_rest.find("\"")?;
    }

    let (val, _) = rest.split_at(pos + rest.len() - temp_rest.len());

    Some(val.to_owned())
}

// 7064392794241068550 -> -398130841357517455 [label="",color="black",fontcolor="black"];
/// Parse link if `line` indeed represents a link
fn try_parse_link(line: &str) -> Option<Link> {

    // find from id
    let pos = line.find(' ')?;
    let (id, rest) = line.split_at(pos);

    let from = id.to_owned();

    // find arrow

    let pos = rest.find("->")?;
    // let (_, rest) = line.split_at(pos);

    if pos == 1 {
        // arrow
    } else {
        return None;
    }

    let (_, rest) = rest.split_at(pos + 3);

    // find to id

    let pos = rest.find(' ')?;

    let (id, rest) = rest.split_at(pos);

    let to = id.to_owned();

    // find label
    let label = find_property(rest, "label").unwrap_or("".to_string());
    let color = find_property(rest, "color").unwrap_or("black".to_string());
    let fontcolor = find_property(rest, "fontcolor").unwrap_or("black".to_string());

    let link = Link {
        from, to, label, color, fontcolor
    };

    Some(link)
}

fn parse_dot(content: &str) -> DotFile {

    let mut nodes = vec![];
    let mut links = vec![];

    let lines : Vec<&str> = content.clone().split('\n').skip(4).collect();

    for line in &lines {

        let first_char = line.chars().next().unwrap();

        if first_char != '{' && first_char != '}' {

            if let Some(link) = try_parse_link(line) {
                links.push(link);
            } else {
                let mut node = parse_node(line);

                // println!("Parsed node: {:?}", node);

                if nodes.len() == 0 {
                    node.is_root = true;
                }
                nodes.push(node);
            }
        }
    }

    DotFile { nodes, links }
}

fn parse_label(label: &str) -> Vec<Assignment> {

    label.split("\\n").map(|token| {

        let pos = token.find(' ').unwrap();

        let (_, rhs) = token.split_at(pos+1);

        let pos = rhs.find(" = ").unwrap();

        let (var, rhs) = rhs.split_at(pos);

        let (_, val) = rhs.split_at(3);

        Assignment {var: var.to_owned(), val: val.to_owned()}

    }).collect()
}

fn find_diff(from: &Node, to: &Node) -> Vec<String> {

    let mut vars = vec![];

    for a in &from.assignments {

        // Find assignment in `to`:

        let a2 = to.assignments.iter().find(|ass| ass.var == a.var).unwrap();

        if a.val != a2.val {
            vars.push(a.var.to_owned());
        }

    }

    vars

}

fn label_links(dotfile: &mut DotFile) {

    // Enter root, find all possible links

    for node in &dotfile.nodes {

        for link in &mut dotfile.links {

            if link.from == node.id {

                let to_node = dotfile.nodes.iter().find(|n| n.id == link.to).unwrap();

                let diff = find_diff(node, &to_node);

                let label = diff.iter().fold(String::new(), |acc, el| {format!("{} {} \n", acc, el)});

                link.label = label.trim_end().to_owned();
            }
        }


    }

    // Next nodes

}

fn change_link_label(content: &String, link: &Link) -> String {

    let lines: Vec<_> = content.split('\n').collect();
    
    let search = format!("{} -> {}", link.from, link.to);

    let mut content: Vec<String> = vec![];

    for line in lines {

        let pos = line.find(&search);

        if pos.is_some() {
            
            let pos = line.find("label=\"\"");

            if pos.is_some() {

                let (lhs, rhs) = line.split_at(pos.unwrap());

                let (_, rhs) = rhs.split_at(8);
    
                let line = format!("{}label=\"{}\"{}", lhs, link.label, rhs);

    
                content.push(line);

            } else {
                content.push(line.to_owned());
            }

        } else {
            content.push(line.to_owned());
        }

    }

    content.join("\n")

}

fn add_links_to_content(content: &String, links: &Vec<Link>) -> String {

    let mut content = content.clone();

    for link in links {
        content = change_link_label(&content, link);
    }
    content

}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    let args : Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: dot_processor <file.dot>");
        std::process::exit(1);
    }

    let filename = &args[1];

    let mut content = std::fs::read_to_string(&filename)?;

    if content.lines().next() == Some("//<processed>") {
        println!("Dot file already processed");
        return Ok(());
    }

    let mut dotfile = parse_dot(&content);

    label_links(&mut dotfile);

    let content = add_links_to_content(&mut content, &dotfile.links);

    // add a marker so we don't attempt to process the file again in the future

    let content = format!("//<processed>\n{}", &content);

    std::fs::write(filename, &content).expect("Could not write to a file");

    Ok(())

}

#[cfg(test)]
mod tests {

    use super::*;

#[test]
fn label_parsing() {

    // let label = "/\\\\ incomingWitnesses = {}\\n/\\\\ quotes = <<>>\\n/\\\\ depositedWitnesses = {[id |-> 0, qid |-> 1]}\\n/\\\\ witnesses = <<[id |-> 0, qid |-> 1]>>\\n/\\\\ refundedWitnesses = {}\\n/\\\\ autoswapEnabledForQuote = <<{FALSE, TRUE}>>\\n/\\\\ executedQuotes = {[qid |-> 1]}\\n/\\\\ incomingQuotes = {}\\n/\\\\ incomingExecReqs = {}";

    let label1 = "/\\ incomingWitnesses = {}\n/\\ quotes = <<[id |-> 1]>>\n/\\ expiredQuotes = {}\n/\\ depositedWitnesses = {}\n/\\ action = \"START\"\n/\\ witnesses = <<[id |-> 0, qid |-> 1]>>\n/\\ refundedWitnesses = {}\n/\\ autoswapEnabledForQuote = <<{FALSE, TRUE}>>\n/\\ executedQuotes = {[qid |-> 1]}\n/\\ incomingQuotes = {}\n/\\ incomingExecReqs = {}";

    let ass1 = parse_label(label1);

    let label2 ="/\\ incomingWitnesses = {}\n/\\ quotes = <<>>\n/\\ expiredQuotes = {[id |-> 1]}\n/\\ depositedWitnesses = {}\n/\\ action = \"Quote expired\"\n/\\ witnesses = <<[id |-> 0, qid |-> 1]>>\n/\\ refundedWitnesses = {[id |-> 0, qid |-> 1]}\n/\\ autoswapEnabledForQuote = <<{FALSE, TRUE}>>\n/\\ executedQuotes = {[id |-> 1]}\n/\\ incomingQuotes = {}\n/\\ incomingExecReqs = {[id |-> 0, qid |-> 1]}";

    let ass2 = parse_label(label2);

    let label3 = "/\\ incomingWitnesses = {[id |-> 0, qid |-> 1]}\n/\\ quotes = <<[id |-> 1]>>\n/\\ expiredQuotes = {}\n/\\ depositedWitnesses = {}\n/\\ action = \"START\"\n/\\ witnesses = <<>>\n/\\ refundedWitnesses = {}\n/\\ autoswapEnabledForQuote = <<{FALSE, TRUE}>>\n/\\ executedQuotes = {}\n/\\ incomingQuotes = {}\n/\\ incomingExecReqs = {[id |-> 0, qid |-> 1]}";

    let ass3 = parse_label(label3);

    let label4 = "/\\ incomingWitnesses = {[id |-> 0, qid |-> 1]}\n/\\ quotes = <<>>\n/\\ expiredQuotes = {}\n/\\ depositedWitnesses = {}\n/\\ action = \"START\"\n/\\ witnesses = <<>>\n/\\ refundedWitnesses = {}\n/\\ autoswapEnabledForQuote = <<{FALSE, TRUE}>>\n/\\ executedQuotes = {}\n/\\ incomingQuotes = {[id |-> 1]}\n/\\ incomingExecReqs = {[id |-> 0, qid |-> 1]}";

    let ass4 = parse_label(label4);

    dbg!(ass4);

}

fn diff_helper(ass1: &str, ass2: &str) -> Vec<String> {

    let n1 = Node {
        id: "0".to_owned(),
        label: "0".to_owned(),
        assignments: parse_label(ass1),
        is_root: false,
    };

    let n2 = Node {
        id: "0".to_owned(),
        label: "0".to_owned(),
        assignments: parse_label(ass2),
        is_root: false,
    };

    dbg!(&n1);
    dbg!(&n2);

    find_diff(&n1, &n2)
}


#[test]
fn finds_diff_in_string_assignments() {

    let n1 = Node {
        id: "0".to_owned(),
        label: "0".to_owned(),
        assignments: vec![Assignment { var: "action".to_string(), val: "\"started\"".to_string()}],
        is_root: false,
    };

    let n2 = Node {
        id: "0".to_owned(),
        label: "0".to_owned(),
        assignments: vec![Assignment { var: "action".to_string(), val: "\"quote expired\"".to_string()}],
        is_root: false,
    };

    let diff = find_diff(&n1, &n2);

    assert_eq!(diff, ["action"]);

    let ass1 = r#"/\\ incomingWitnesses = {[id |-> 0, qid |-> 1]}\n/\\ quotes = <<[id |-> 1]>>\n/\\ expiredQuotes = {}\n/\\ depositedWitnesses = {}\n/\\ action = \"START\"\n/\\ witnesses = <<>>\n/\\ refundedWitnesses = {}\n/\\ autoswapEnabledForQuote = <<{FALSE, TRUE}>>\n/\\ executedQuotes = {}\n/\\ incomingQuotes = {}\n/\\ incomingExecReqs = {[id |-> 0, qid |-> 1]}"#;
    let ass2 = r#"/\\ incomingWitnesses = {[id |-> 0, qid |-> 1]}\n/\\ quotes = <<[id |-> 1]>>\n/\\ expiredQuotes = {}\n/\\ depositedWitnesses = {}\n/\\ action = \"START\"\n/\\ witnesses = <<>>\n/\\ refundedWitnesses = {}\n/\\ autoswapEnabledForQuote = <<{FALSE, TRUE}>>\n/\\ executedQuotes = {[qid |-> 1]}\n/\\ incomingQuotes = {}\n/\\ incomingExecReqs = {}"#;

    let res = diff_helper(ass1, ass2);

    assert_eq!(res, ["executedQuotes", "incomingExecReqs"]);


}

#[test]
fn correctly_finds_property_a() {

    let line = r#"xxx label=" a=\"Hello\" ""#;

    let prop = find_property(line, "label");

    assert_eq!(prop, Some(" a=\\\"Hello\\\" ".to_string()));

}

#[test]
fn correctly_finds_property_b() {

    let line = " [label=\"\",color=\"black\",fontcolor=\"black\"];";

    let prop = find_property(line, "label");

    assert_eq!(prop, Some("".to_string()));

}

}