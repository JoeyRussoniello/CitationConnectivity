use crate::graph::*;
pub fn mark_component_bfs(vertex:Vertex,graph:&Graph,component:&mut Vec<Option<Component>>, component_no:Component){
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

pub fn count_components(component:&Vec<Option<Component>>,num_components:usize) -> Vec<usize>{
    //Get the count of nodes in each component
    let mut component_counts = vec![0;num_components];
    //Component counts from BFS are 1 indexed, but  unwrap() - 1 adjusts to a 0-index
    component.iter().for_each(|x| component_counts[x.unwrap() -1] += 1);
    return component_counts;
}
pub fn get_component_scale(component:&Vec<Option<Component>>, num_components:usize,sort:bool) -> Vec<f64>{
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