use std::collections::{HashMap,VecDeque}; //HashMap used to map node IDs, VecDeQueue used for BFS
use plotters::prelude::*;
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
struct Graph{
    n: usize,
    outedges: AdjacencyList,
    node_data:HashMap<usize,NodeData>,
    reverse_map:HashMap<usize,usize>
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
    fn from_csvs(edge_path:&str, node_path:&str) -> Result<Self,String>{
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
}
fn main() {
    let graph = Graph::from_csvs("citation_network\\edges.csv","citation_network\\nodes.csv").unwrap();

    let mut component:Vec<Option<Component>> = vec![None;graph.n];
    let mut component_count = 0;
    for v in 0..graph.n{
        if let None = component[v]{
            component_count += 1;
            mark_component_bfs(v, &graph, &mut component, component_count);
        }
    }
    
    let component_scale = get_component_scale(&component, component_count + 1);
    create_svg(&component_scale, "plots\\component_aggregation.svg").expect("Error in Image Creation");
}

fn mark_component_bfs(vertex:Vertex,graph:&Graph,component:&mut Vec<Option<Component>>, component_no:Component){
    component[vertex] = Some(component_no);

    let mut queue = VecDeque::new();
    queue.push_back(vertex);

    while let Some(v) = queue.pop_front(){
        for u in graph.outedges[v].iter(){
            //If not visited
            if let None = component[*u]{
                component[*u] = Some(component_no);
                queue.push_back(*u);
            }
        }
    }
}

fn count_components(component:&Vec<Option<Component>>,num_components:usize) -> Vec<usize>{
    //Get the count of nodes in the most components in descending order
    let mut component_counts = vec![0;num_components];
    component.iter().for_each(|x| component_counts[x.unwrap()] += 1);
    component_counts.sort();
    component_counts.reverse();
    return component_counts;
}
fn get_component_scale(component:&Vec<Option<Component>>, num_components:usize) -> Vec<f64>{
    let component_counts = count_components(component, num_components);
    let mut aggregate_component_counts = vec![0.0;num_components + 1];
    let total = component_counts.iter().fold(0,|acc, x| acc + x) as f64;
    let mut running_sum = 0;
    for (index, elem) in component_counts.iter().enumerate(){
        running_sum += elem;
        let aggregate = running_sum as f64 / total;
        aggregate_component_counts[index + 1] = aggregate;
    }
    return aggregate_component_counts
}
fn create_svg(points:&Vec<f64>,filename:&str) -> Result<(), Box<dyn std::error::Error>> {
    // Create the drawing area, using the provided filename
    let root = SVGBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    // Change Title Name Later
    let root_area = root.titled("Connectivity Progress by Component", ("sans-serif", 60))?;
    
    // Create the chart builder with custom x-axis range (allowing negative values)
    let mut cc = ChartBuilder::on(&root_area)
        .margin(5)
        .set_all_label_area_size(50)
        .build_cartesian_2d(-5..points.len() as isize, 0.0..1.05)?;  // Adjust x-range to allow negative values

    // Configure mesh and labels for y-axis
    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .y_label_formatter(&|v: &f64| format!("{:.1}", v))
        .draw()?;

    // Draw the line series
    cc.draw_series(LineSeries::new(
        points.iter().enumerate().map(|(index, &value)| {
            (index as isize, value) // Map x to allow negative values
        }),
        &BLUE,
    ))?;

    Ok(())
}