use core::num;
use std::collections::{HashMap,VecDeque};  //HashMap used to map node IDs, VecDeQueue used for BFS
use plotters::prelude::*; //For plotting visuals
use full_palette::{LIGHTBLUE_100};
use rand::Rng; // For random placements within component clusters 
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
    fn connected_components(&self) -> (Vec<Option<Component>>, usize){
        let mut component:Vec<Option<Component>> = vec![None;self.n];
        let mut component_count = 0;
        for v in 0..self.n{
            if let None = component[v]{
                component_count += 1;
                mark_component_bfs(v, self, &mut component, component_count);
            }
        }
        return (component, component_count + 1)
    }
}
fn main() {
    let graph = Graph::from_csvs("citation_network\\edges.csv","citation_network\\nodes.csv").unwrap();

    graph.connected_components();
    
    let (component,num_components) = graph.connected_components();
    let component_sizes = count_components(&component, num_components);
    let component_scale = get_component_scale(&component, num_components,true);

    println!("Component Sizes: {:?}",component_sizes);
    println!("Component Scale: {:?}",component_scale);
    //Plot the component effectiveness
    create_svg(&component_scale, "plots\\component_aggregation.png").expect("Error in Aggregate Image Creation");
    //REALLY cool custom visual that shows the connectivity of the graph components
    visualize_graph("plots\\connected_components.png", &graph.outedges, &component,num_components,3.5).expect("Error in Connected Image Creation");
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
    //Get the count of nodes in each component
    let mut component_counts = vec![0;num_components];
    component.iter().for_each(|x| component_counts[x.unwrap()] += 1);
    return component_counts;
}
fn get_component_scale(component:&Vec<Option<Component>>, num_components:usize,sort:bool) -> Vec<f64>{
    //Calculates in descending order the aggregate % of data encapsulated by the largest components 
    let mut component_counts = count_components(component, num_components);
    if sort{
        component_counts.sort();
        component_counts.reverse();
    }
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
    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    // Change Title Name Later
    let root_area = root.titled("Connectivity Progress by Component", ("sans-serif", 40))?;
    
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

fn get_graph_dimensions(
    component: &Vec<Option<usize>>, 
    num_components: usize, 
    drawing_bounds: (i32, i32, i32, i32),
    biggest_circle: f64,
    grid_size: f64, // Size of each cell in the grid
) -> HashMap<usize, ((f64, f64), f64)> {
    let (x_min, x_max, y_min, y_max) = drawing_bounds;
    let component_counts = count_components(component, num_components);
    let total: usize = component_counts.iter().sum();
    let mut rng = rand::thread_rng();
    let mut component_sorted_tuples: Vec<(usize, usize)> = component_counts
        .iter()
        .enumerate()
        .map(|(index, value)| (index, *value))
        .collect();
    component_sorted_tuples.sort_by(|a, b| b.1.cmp(&a.1));

    let mut current_x = x_min as f64;
    let total_x_space = (x_max - x_min) as f64;

    let mut current_y = y_min as f64;
    let total_y_space = (y_max - y_min) as f64;

    let mut range_hash = HashMap::<usize, ((f64, f64), f64)>::new();
    let mut grid: HashMap<(usize, usize), Vec<(f64, f64, f64)>> = HashMap::new();

    for (index, value) in component_sorted_tuples {
        let percentage_needed = value as f64 / total as f64;

        // Calculate radius based on percentage size
        let min_radius = 50.0;
        let max_radius = (total_x_space.min(total_y_space)) / biggest_circle;
        let circle_radius = (percentage_needed.sqrt() * max_radius).max(min_radius);

        // Determine cluster center
        let mut center_x = current_x + circle_radius;
        let mut center_y = current_y + circle_radius;

        // Spatial checking to resolve overlap
        let mut overlap = true;
        while overlap {
            overlap = false;

            // Check positions in the grid for potential overlap
            let cell_x = (center_x / grid_size).floor() as usize;
            let cell_y = (center_y / grid_size).floor() as usize;

            if let Some(neighbors) = grid.get(&(cell_x, cell_y)) {
                for &(placed_x, placed_y, placed_radius) in neighbors {
                    let distance = ((center_x - placed_x).powi(2) + (center_y - placed_y).powi(2)).sqrt();
                    if distance < (circle_radius + placed_radius) {
                        overlap = true;
                        break;
                    }
                }
            }

            if overlap {
                // Randomly adjust position if there was an overlap
                center_x = rng.gen_range(x_min as f64..x_max as f64);
                center_y = rng.gen_range(y_min as f64..y_max as f64);
            }
        }

        // Store the new position and radius in the hash map
        range_hash.insert(index, ((center_x, center_y), circle_radius));
        let cell_x = (center_x / grid_size).floor() as usize;
        let cell_y = (center_y / grid_size).floor() as usize;
        grid.entry((cell_x, cell_y)).or_default().push((center_x, center_y, circle_radius));

        // Move horizontally for the next component with jitter
        current_x += circle_radius * rng.gen_range(2.0..4.0);

        // Step down the Y-axis dynamically for the next component with jitter
        current_y += circle_radius * rng.gen_range(2.0..3.0) * (1.0 + (1.0 - percentage_needed.sqrt()));

        // Check bounds
        if current_x + circle_radius > x_max as f64 {
            current_x = x_min as f64 + rng.gen_range(0.0..100.0);
        }

        if current_y + circle_radius > y_max as f64 {
            current_y = y_min as f64 + rng.gen_range(0.0..100.0);
        }
    }

    range_hash
}


fn interpolate_color(start_color: (u8, u8, u8), end_color: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let (r_start, g_start, b_start) = start_color;
    let (r_end, g_end, b_end) = end_color;

    let r = (r_start as f64 + t * (r_end as f64 - r_start as f64)) as u8;
    let g = (g_start as f64 + t * (g_end as f64 - g_start as f64)) as u8;
    let b = (b_start as f64 + t * (b_end as f64 - b_start as f64)) as u8;

    (r, g, b)
}

fn get_color_from_gradient(index: usize, total: usize) -> RGBAColor {
    // Define the start and end colors (dark blue to teal)
    let dark_blue = (0, 0, 139);  // RGB for dark blue
    let teal = (0, 128, 128);    // RGB for teal

    // Normalize the index to a value between 0.0 and 1.0
    let t = index as f64 / (total - 1) as f64;

    // Interpolate the color based on the normalized value
    let (r,g,b) = interpolate_color(dark_blue, teal, t);
    RGBAColor(r,g,b,1.0)
}

fn visualize_graph(
    output_file: &str,
    adjacency_list: &Vec<Vec<usize>>,
    components: &Vec<Option<usize>>,
    num_components: usize,
    biggest_circle: f64
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_file, (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    // Title for the graph
    let root_area = root.titled("Component Connectivity", ("sans-serif", 40))?;

    // Determine the bounds of the drawing area
    let (x_min, x_max, y_min, y_max) = (-500, 500, -500, 500);

    let mut cc = ChartBuilder::on(&root_area)
        .margin(10)
        .caption("Distribution of citation network by connected component", ("sans-serif", 20))
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    cc.configure_mesh().disable_mesh().draw()?;

    // Step 1: Assign positions to nodes, clustered by components
    let range_hash = get_graph_dimensions(components, num_components, (-500, 500, -500, 500), biggest_circle,50.0);
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
    for (node, neighbors) in adjacency_list.iter().enumerate() {
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
            let color = get_color_from_gradient(component,num_components);

            // Draw the node with the assigned color
            cc.draw_series(std::iter::once(Circle::new((x, y), 5, color.filled())))?;
        }
    }

    Ok(())
}

