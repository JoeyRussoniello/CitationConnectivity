pub mod component_functions;
pub mod visualization_support;

use std::collections::{HashMap,VecDeque};
use plotters::prelude::*;
use rand::Rng;
type Vertex = usize;
type Edge = (Vertex, Vertex);
type AdjacencyList = Vec<Vec<Vertex>>;
type Component = usize;

#[derive(Debug,Clone)]
#[allow(dead_code)] //Allowed for now until ML features get developed
pub struct NodeData{
    pub mapped_node:usize,
    pub label:String,
    pub subject:String,
    pub features:Vec<u8>
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
    
    pub fn calculate_subgraphs(self) -> Vec<(String,Self)>{
        //RETHINK THIS APPROACH CITATION CIRCLES WON'T WORK HERE

        let mut subject_graph_info = HashMap::<String, (usize, AdjacencyList, HashMap<usize, NodeData>, HashMap<usize,usize>)>::new();

        //For each point in the grpah
        for i in 0..self.n{
            let original_node = self.reverse_map.get(&i).unwrap(); //Get the original node
            let node_data = self.node_data.get(&original_node).unwrap(); //Get the original node's data
            let subject = node_data.subject.clone(); //Get the subject of the original node
            let mut outedge = self.outedges[i].clone(); //Get the edges from the mapped_node
            outedge = outedge.iter().map(|edge| *self.reverse_map.get(edge).unwrap()).collect();
            //init a new entry or get the existing entry from the subject hash
            let existing_subject_data = subject_graph_info.entry(subject).or_insert((0,vec![],HashMap::new(),HashMap::new())); 
            existing_subject_data.0 += 1; //Count 1 more vertex
            existing_subject_data.1.push(outedge); //Append Adjacency list (of original nodes)
            existing_subject_data.3.insert(existing_subject_data.0 - 1, *original_node); //Guide to reverse the mapping in the new graph

            let mut adjusted_node_data = node_data.clone();
            adjusted_node_data.mapped_node = existing_subject_data.0 -1;
            existing_subject_data.2.insert(*original_node,adjusted_node_data); //Node Data
        }

        //Hash search a subject and get the nodes in that subject
        let mut output = Vec::new();
        for (subject, (n, adj_list, node_data,reverse_map)) in subject_graph_info{
            let mut adjusted_outedges:Vec<Vec<usize>> = Vec::new();
            for outedge in adj_list.iter(){
                let mut adjusted_outedge:Vec<usize> = Vec::new();
                for edge in outedge{
                    if node_data.contains_key(&edge){
                        adjusted_outedge.push(node_data.get(&edge).unwrap().mapped_node);
                    }
                }
                adjusted_outedges.push(adjusted_outedge);
            }
            output.push((subject,Graph{n,outedges:adjusted_outedges,node_data,reverse_map}));
        }
        return output
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
    //Create a png graph of the connected components of the graph
    pub fn visualize_connectivity(
        &self,
        output_file: &str,
        biggest_circle: f64,
        output_size:(u32,u32),
        title:&str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_file, output_size).into_drawing_area();
        root.fill(&WHITE)?;
    
        // Title for the graph
        let root_area = root.titled(title, ("sans-serif", 40))?;
    
        // Determine the bounds of the drawing area
        let drawing_area = (-500,500,-500,500);
        let (x_min, x_max, y_min, y_max) = drawing_area;

        let mut cc = ChartBuilder::on(&root_area)
            .margin(10)
            .caption("Distribution of citation network by connected component", ("sans-serif", 20))
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    
        cc.configure_mesh().disable_mesh().draw()?;
    
        //Run algorithm
        let (components,num_components) = self.connected_components();
        
        // Step 1: Assign positions to nodes, clustered by components
        let range_hash = visualization_support::get_graph_dimensions(&components, num_components, drawing_area, biggest_circle,50.0);
        let mut rng = rand::thread_rng();
        let mut positions: HashMap<usize, (i32, i32)> = HashMap::new();
    
        for (i, component) in components.iter().enumerate() {
            if let Some(comp) = component {
                if let Some((center, radius)) = range_hash.get(comp) {
                    let (center_x, center_y) = *center;
    
                    // Generate random position within the circle using polar coordinates
                    let random_angle = rng.gen_range(0.0..(2.0 * std::f64::consts::PI)); // Random angle
                    let random_radius = rng.gen_range(0.0..=*radius); // Random radius (uniform distribution)
    
                    let offset_x = random_radius * random_angle.cos();
                    let offset_y = random_radius * random_angle.sin();
    
                    let node_x = (center_x + offset_x) as i32;
                    let node_y = (center_y + offset_y) as i32;
                    positions.insert(i, (node_x, node_y));
                }
            }
        }
    
        // Step 2: Draw edges between nodes
        for (node, neighbors) in self.outedges.iter().enumerate() {
            if let Some(&(x1, y1)) = positions.get(&node) {
                for &neighbor in neighbors {
                    if let Some(&(x2, y2)) = positions.get(&neighbor) {
                        cc.draw_series(LineSeries::new(vec![(x1, y1), (x2, y2)], &full_palette::CYAN_100))?;
                    }
                }
            }
        }
    
        // Step 3: Draw nodes as circles with color based on their component
        for (node, &(x, y)) in positions.iter() {
            if let Some(component) = components[*node] {
                // Get the color for the component
                let color = visualization_support::get_color_from_gradient(component,num_components);
    
                // Draw the node with the assigned color
                cc.draw_series(std::iter::once(Circle::new((x, y), 5, color.filled())))?;
            }
        }
    
        Ok(())
    }
}
