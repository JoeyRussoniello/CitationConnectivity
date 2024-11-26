pub mod component_functions;
pub mod visualization_support;

use std::collections::{HashMap,VecDeque};

type Vertex = usize;
type Edge = (Vertex, Vertex);
type AdjacencyList = Vec<Vec<Vertex>>;
type Component = usize;

#[derive(Debug,Clone)]
#[allow(dead_code)] //Allowed for now until ML features get developed
struct NodeData{
    mapped_node:usize,
    label:String,
    subject:String,
    features:Vec<u8>
}
//Function for NodeData to read from csv
impl NodeData{
    //Serialize the input String record
    fn read_strings(line:&csv::StringRecord,mapped_node:usize) -> Self{
        let label = String::from(&line[2]);
        let subject = String::from(&line[3]);
        let features: Vec<u8> = line[4]
            .trim_matches(|c| c == '[' || c == ']') // Remove the brackets
            .split(',')                            // Split by comma
            .filter_map(|x| x.parse::<u8>().ok()) // Parse to usize and filter out invalid entries
            .collect();                            // Collect into a vector
        NodeData{
            mapped_node,
            label,
            subject,
            features
        }
    }
}
#[derive(Debug)]
pub struct Graph{
    pub n: usize,
    pub outedges: AdjacencyList,
    pub node_data:HashMap<usize,NodeData>,
    pub reverse_map:HashMap<usize,usize>
}
impl Graph{
    //Create graph from directed edges
    fn create_directed(n: usize, edges: &Vec<Edge>,node_data:HashMap<usize,NodeData>,reverse_hash:HashMap<usize,usize>) -> Self{
        let mut adj_list:AdjacencyList = vec![vec![];n];
        for (v, w) in edges.iter(){
            adj_list[*v].push(*w);
        }
        //We allow the data to be moved here, since we want it to live in the object anyways
        Graph{n,outedges:adj_list,node_data:node_data,reverse_map:reverse_hash}
    }
    //Read the input csv files
    pub fn from_csvs(edge_path:&str, node_path:&str) -> Result<Self,String>{
        let mut node_rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_path(node_path).expect("Expected a valid file path");
        //Iterate over each record, creating a corresponding map for it
        let mut node_data= HashMap::<usize,NodeData>::new(); //Can be used to search a node and get data
        let mut reverse_hash = HashMap::<usize,usize>::new(); //Can be used to undo the node mapping
        for (index, record) in node_rdr.records().enumerate(){
            let r = record.expect("A CSV Line In node_path");
            let data = NodeData::read_strings(&r, index);
            let node_id = String::from(&r[1]).parse::<usize>().unwrap();
            node_data.insert(node_id,data);
            reverse_hash.insert(index, node_id);
        }

        let mut edge_rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_path(edge_path).expect("Expected a valid file path");
        let mut edges:Vec<Edge> = Vec::new();
        for record in edge_rdr.records(){
            let r = record.expect("A CSV Line in edge_path");
            //Get the node_id and its mapped index, raise an error if not found
            let node_id = String::from(&r[1]).parse::<usize>().unwrap();
            let node_id_adjusted = match node_data.get(&node_id){
                Some(node) => node.mapped_node,
                None => return Err(format!("Node ID {} not found in node data",node_id).into())
            };

            //Get the target_id and its mapped index, raise an error if not found
            let target_id = String::from(&r[2]).parse::<usize>().unwrap();
            let target_id_adjusted = match node_data.get(&target_id){
                Some(node) => node.mapped_node,
                None => return Err(format!("Target Node ID {} not found in node data",target_id).into())
            };

            edges.push((node_id_adjusted,target_id_adjusted));
        }
        
        return Ok(Graph::create_directed(node_data.len(),&edges,node_data,reverse_hash));
    }
    pub fn connected_components(&self) -> (Vec<Option<Component>>, usize){
        let mut component:Vec<Option<Component>> = vec![None;self.n];
        let mut component_count = 0;
        for v in 0..self.n{
            if let None = component[v]{
                component_count += 1;
                component_functions::mark_component_bfs(v, self, &mut component, component_count);
            }
        }
        return (component, component_count + 1)
    }
}
