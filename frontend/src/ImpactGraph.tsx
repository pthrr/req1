import { useEffect, useRef } from "react";
import * as d3 from "d3";
import type { ImpactResponse } from "./api/client";
import { theme } from "./theme";

interface Props {
  response: ImpactResponse;
  rootId: string;
}

interface GraphNode extends d3.SimulationNodeDatum {
  id: string;
  label: string;
  depth: number;
  isRoot: boolean;
}

interface GraphLink extends d3.SimulationLinkDatum<GraphNode> {
  linkType: string | null;
  suspect: boolean;
}

const DEPTH_COLORS = ["#1976d2", "#388e3c", "#f57c00", "#d32f2f", "#7b1fa2", "#00796b"];

export function ImpactGraph({ response, rootId }: Props) {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    const width = svgRef.current.clientWidth || 700;
    const height = svgRef.current.clientHeight || 450;

    // Build nodes
    const nodeMap = new Map<string, GraphNode>();

    // Root node
    nodeMap.set(rootId, {
      id: rootId,
      label: rootId.slice(0, 8),
      depth: 0,
      isRoot: true,
    });

    for (const obj of response.objects) {
      if (!nodeMap.has(obj.id)) {
        nodeMap.set(obj.id, {
          id: obj.id,
          label: obj.heading ?? obj.level,
          depth: obj.depth,
          isRoot: false,
        });
      }
    }

    const nodes = Array.from(nodeMap.values());

    // Build links from edges
    const links: GraphLink[] = response.edges
      .filter((e) => nodeMap.has(e.source_id) && nodeMap.has(e.target_id))
      .map((e) => ({
        source: e.source_id,
        target: e.target_id,
        linkType: e.link_type,
        suspect: e.suspect,
      }));

    // Simulation
    const simulation = d3
      .forceSimulation<GraphNode>(nodes)
      .force(
        "link",
        d3
          .forceLink<GraphNode, GraphLink>(links)
          .id((d) => d.id)
          .distance(100),
      )
      .force("charge", d3.forceManyBody().strength(-300))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collision", d3.forceCollide(30));

    // Arrow marker
    svg
      .append("defs")
      .append("marker")
      .attr("id", "arrowhead")
      .attr("viewBox", "0 -5 10 10")
      .attr("refX", 25)
      .attr("refY", 0)
      .attr("markerWidth", 6)
      .attr("markerHeight", 6)
      .attr("orient", "auto")
      .append("path")
      .attr("d", "M0,-5L10,0L0,5")
      .attr("fill", "#999");

    // Links
    const linkGroup = svg
      .append("g")
      .selectAll<SVGLineElement, GraphLink>("line")
      .data(links)
      .join("line")
      .attr("stroke", (d) => (d.suspect ? "#f44336" : "#999"))
      .attr("stroke-width", 1.5)
      .attr("stroke-dasharray", (d) => (d.suspect ? "5,3" : "none"))
      .attr("marker-end", "url(#arrowhead)");

    // Link labels
    const linkLabels = svg
      .append("g")
      .selectAll<SVGTextElement, GraphLink>("text")
      .data(links.filter((l) => l.linkType))
      .join("text")
      .attr("font-size", "9px")
      .attr("fill", theme.colors.textSecondary)
      .attr("text-anchor", "middle")
      .text((d) => d.linkType ?? "");

    // Nodes
    const nodeGroup = svg
      .append("g")
      .selectAll<SVGGElement, GraphNode>("g")
      .data(nodes)
      .join("g")
      .call(
        d3
          .drag<SVGGElement, GraphNode>()
          .on("start", (event, d) => {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            d.fx = d.x;
            d.fy = d.y;
          })
          .on("drag", (event, d) => {
            d.fx = event.x;
            d.fy = event.y;
          })
          .on("end", (event, d) => {
            if (!event.active) simulation.alphaTarget(0);
            d.fx = null;
            d.fy = null;
          }),
      );

    nodeGroup
      .append("circle")
      .attr("r", (d) => (d.isRoot ? 14 : 10))
      .attr("fill", (d) => DEPTH_COLORS[d.depth % DEPTH_COLORS.length])
      .attr("stroke", "#fff")
      .attr("stroke-width", 2);

    nodeGroup
      .append("text")
      .attr("dy", -16)
      .attr("text-anchor", "middle")
      .attr("font-size", "11px")
      .attr("fill", theme.colors.text)
      .text((d) => (d.label.length > 20 ? d.label.slice(0, 18) + "..." : d.label));

    // Tick
    simulation.on("tick", () => {
      linkGroup
        .attr("x1", (d) => ((d.source as GraphNode).x ?? 0))
        .attr("y1", (d) => ((d.source as GraphNode).y ?? 0))
        .attr("x2", (d) => ((d.target as GraphNode).x ?? 0))
        .attr("y2", (d) => ((d.target as GraphNode).y ?? 0));

      linkLabels
        .attr("x", (d) => (((d.source as GraphNode).x ?? 0) + ((d.target as GraphNode).x ?? 0)) / 2)
        .attr("y", (d) => (((d.source as GraphNode).y ?? 0) + ((d.target as GraphNode).y ?? 0)) / 2 - 6);

      nodeGroup.attr("transform", (d) => `translate(${d.x ?? 0},${d.y ?? 0})`);
    });

    return () => {
      simulation.stop();
    };
  }, [response, rootId]);

  return (
    <svg
      ref={svgRef}
      style={{ width: "100%", height: 450, border: `1px solid ${theme.colors.borderLight}`, borderRadius: theme.borderRadius }}
    />
  );
}
