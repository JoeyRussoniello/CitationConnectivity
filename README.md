# CitationConnectivity
# Table of Contents
1. [Project Description](#project-description)
2. [Features](#features)
3. [Installation](#installation)
4. [Usage](#usage)
5. [Output](#output)
6. [Contribution](#contribution)
## Project Description
CitationConnectivity is a Rust project that analyzes and visualizes the connectivity of a citation network (example provided [from the CORA citations dataset](https://graphsandnetworks.com/the-cora-dataset/)). It processes citation data to identify connected components, visualize their structure, and generate subgraph statistics for different research subjects. The project is designed to work with CSV files containing nodes (representing research papers) and edges (representing citations between them).

## Features
- **Connected Components Analysis**:Identify and analyze the connected components of a citation network.
- **Component Visualization**: Generate visualizations for the overall network and its subgraphs to represent connectivity patterns.
- **Subgraph Analysis**: Break down the network by research subjects and calculate connectivity statistics for each subgraph.
  - *In subgraph creation, both the original nodes of the graph and the edges in the adjacency list are modified to only include vertices within the specified field*
- **Customizable Visualizations**: Supports creating tailored plots for understanding connectivity and aggregation of components.
## Installation
### Dependencies
- [Rust](https://www.rust-lang.org/) (latest stable version)
- `csv` crate for reading CSV files
- `plotters` crate for generating visualizations
- `rand` crate for random number generation
### Steps
1. Clone this repository:
```bash
git clone https://github.com/your-repo/CitationConnectivity.git
cd CitationConnectivity
```
2. Build the project:
```bash
cargo build
```
3. Run the project:
```bash
bash
cargo run
```

### File Structure
```bash
CitationConnectivity/
├── src/
│   ├── main.rs                # Entry point for the application
│   ├── graph.rs               # Main module for graph structure and operations
│   ├── component_functions.rs # Utility functions for analyzing components
│   ├── visualization_support.rs # Support functions for graph visualization
├── citation_network/
│   ├── edges.csv              # Example input file containing citation edges
│   ├── nodes.csv              # Example input file containing node metadata
├── plots/
│   ├── connected_components.png # Output visualization of connected components
│   ├── subgraphs/             # Visualizations of subgraph connectivity
└── Cargo.toml                 # Project dependencies
```
## Usage
Input Files
The project reads two CSV files from [the CORA citations dataset](https://graphsandnetworks.com/the-cora-dataset/):

nodes.csv: Contains metadata for each node, including:
- Node ID
- Label
- Subject
- Features (A one-hot encoded list indicating the presence of common words in the paper)

edges.csv: Contains directed edges between nodes, specifying the citation network.
### Main Function
The main.rs function loads the graph, computes connected components, visualizes them, and generates subgraph statistics. Example usage:

```rust
//Create Graph From input CSV files
let graph = Graph::from_csvs("citation_network\\edges.csv","citation_network\\nodes.csv").unwrap();

//Divide graph by research subject
let subgraphs = graph.calculate_subgraphs();

//Plot each research subject's self-connectivity
subgraphs.iter().for_each(|(subject, subgraph)| {
    subgraph.visualize_connectivity(
        &format!("plots\\subgraphs\\{}_connectivity.png",subject), //File path of output image
        3.0,  //The Biggest Circle will only take up 1/3.0 of the graph's total space
        (1024,1024), //Build graph on a 1024 x 1024 canvas
        &format!("Connectivity of Research Papers in {}",subject)).unwrap(); //Graph Title
});
```
## Output
- Component Sizes: Prints the sizes of connected components.
- Visualizations: Generates plots for the overall network and subgraphs in the `plots/` directory.

#### Customization
You can modify the visualization parameters (e.g., plot dimensions, circle sizes) in the visualization_support module.

#### Visualizations Supported
- Connected Components: A visualization of all connected components in the network.
- Subgraph Connectivity: Visualizations for individual research subjects to understand their connectivity.
- Component Progress: A line graph of the aggregate % of data captured in each of the largest components
#### Example Visualizations
See below visualizations of citation networks by research genre, displaying how a papers genre may impact its connectivity within its field.
<p align="center">
  <img src="./plots/subgraphs/Rule_Learning_connectivity.png" width="49%" height="auto" alt="Example Graph for Rule Learning Connectivity">
  <img src="./plots/subgraphs/Genetic_Algorithms_connectivity.png" width="49%" height="auto" alt="Example Graph for Genetic Algorithms Connectivity">
</p>




## Contribution
Feel free to contribute to CitationConnectivity! Here’s how:

Fork the repository.
Create a new branch:
```bash
git checkout -b feature/your-feature-name
```
Commit your changes:
```bash
git commit -m "Add your feature description"
```
Push to the branch:
```bash
git push origin feature/your-feature-name
```
Open a pull request.
