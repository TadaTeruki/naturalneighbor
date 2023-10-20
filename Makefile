# flamegraph is a tool to visualize the performance.
# You need to setup "flamegraph" and "perf" for your system.
# flamegraph: $ cargo install flamegraph
# perf: see https://perf.wiki.kernel.org/index.php/Main_Page 
.PHONY: flamegraph
flamegraph:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --example color