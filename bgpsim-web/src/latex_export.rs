// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::ops::Deref;

use bgpsim::prelude::BgpSessionType;
use itertools::Itertools;

use crate::net::Net;

const LATEX_TEMLPATE: &str = r"
% This file was automatically generated by Bgpsim
\documentclass{standalone}

% latex packages
\usepackage{tikz}
\usetikzlibrary{positioning, arrows, shapes, calc}

% color definitions
\usepackage{xcolor}
\definecolor{gray-50}{HTML}{F9FAFB}
\definecolor{gray-300}{HTML}{D1D5DB}
\definecolor{gray-700}{HTML}{374151}
\definecolor{red-500}{HTML}{EF4444}
\definecolor{yellow-500}{HTML}{EAB308}
\definecolor{green-500}{HTML}{22C55E}
\definecolor{blue-500}{HTML}{3B82F6}
\definecolor{purple-500}{HTML}{A855F7}

% Parameters to edit
\def\width{8}%cm
\def\height{-6}%cm (negative)
\def\linkweightdist{0.3}

% tikzset styles
\tikzset{
  router/.style = {circle, fill=gray-50, draw=gray-700, minimum size=0.4cm},
  external/.style = {circle, fill=gray-300, draw=gray-700, minimum size=0.4cm},
  link/.style = {gray-700},
  next hop/.style = {very thick, -latex, blue-500},
  ebgp session/.style = {very thick, -latex, red-500},
  ibgp peer session/.style = {very thick, latex-latex, blue-500},
  ibgp client session/.style = {very thick, -latex, purple-500},
  bgp propagation/.style = {very thick, -latex, yellow-500},
  link weight/.style = {fill=white},
}

% things to draw
\def\showNextHop{1}
% \def\showLinkWeights{1}
% \def\showBgpSessions{1}
% \def\showBgpPropagation{1}
% \def\showRouterName{1}
\def\prefix1{1} % choices: {{PREFIXES}}

\begin{document}
\begin{tikzpicture}[xscale=\width, yscale=\height]
{{INTERNAL_NODES}}
{{EXTERNAL_NODES}}

{{EDGES}}

  \ifdefined\showNextHop
{{NEXT_HOPS}}
  \fi

  \ifdefined\showLinkWeights
{{LINK_WEIGHTS}}
  \fi

  \ifdefined\showBgpSessions
{{BGP_SESSIONS}}
  \fi

  \ifdefined\showBgpPropagation
{{BGP_PROPAGATIONS}}
  \fi
\end{tikzpicture}
\end{document}
";

pub fn generate_latex(net: &Net) -> String {
    let net_deref = net.net();
    let pos_deref = net.pos_ref();
    let p = pos_deref.deref();
    let n = net_deref.deref();
    let g = n.get_topology();

    let prefix_choices = n
        .get_known_prefixes()
        .map(|p| format!("prefix{}", p.to_string().replace(['.', '/'], "_"),))
        .join(", ");

    let internal_nodes = n
        .get_routers()
        .iter()
        .map(|r| {
            (
                r,
                p.get(r).cloned().unwrap_or_default(),
                n.get_router_name(*r).unwrap_or_default().to_string(),
            )
        })
        .map(|(r, p, n)| {
            format!(
                r"  \node[router] at ({}, {}) (r{}) {{}}; % {}",
                p.x,
                p.y,
                r.index(),
                n
            )
        })
        .join("\n");

    let external_nodes = n
        .get_external_routers()
        .iter()
        .map(|r| {
            (
                r,
                p.get(r).cloned().unwrap_or_default(),
                n.get_router_name(*r).unwrap_or_default().to_string(),
            )
        })
        .map(|(r, p, n)| {
            format!(
                r"  \node[external] at ({}, {}) (r{}) {{}}; % {}",
                p.x,
                p.y,
                r.index(),
                n
            )
        })
        .join("\n");

    let edges = g
        .edge_indices()
        .filter_map(|e| g.edge_endpoints(e))
        .filter(|(a, b)| a.index() < b.index())
        .map(|(a, b)| format!(r"  \draw[link] (r{}) -- (r{});", a.index(), b.index()))
        .join("\n");

    let next_hops = n
        .get_known_prefixes()
        .map(|p| {
            format!(
                "    \\ifdefined\\prefix{}\n{}\n  \\fi",
                p.to_string().replace(['.', '/'], "_"),
                n.get_routers()
                    .into_iter()
                    .filter_map(|r| n.get_device(r).internal())
                    .flat_map(|r| r.get_next_hop(*p).into_iter().map(|nh| (r.router_id(), nh)))
                    .map(|(src, dst)| format!(
                        r"      \draw[next hop] (r{}) -- (r{});",
                        src.index(),
                        dst.index()
                    ))
                    .join("\n")
            )
        })
        .join("\n");

    let link_weights = g
        .edge_indices()
        .map(|e| (g.edge_endpoints(e).unwrap(), g.edge_weight(e).unwrap()))
        .map(|((src, dst), weight)| {
            format!(
                r"    \draw ($(r{})!\linkweightdist!(r{})$) node[link weight] {{ {:.0} }};",
                src.index(),
                dst.index(),
                weight
            )
        })
        .join("\n");

    let bgp_sessions = net
        .get_bgp_sessions()
        .into_iter()
        .map(|(src, dst, ty)| {
            format!(
                r"    \draw[{}] (r{}) to[bend left=20] (r{});",
                match ty {
                    BgpSessionType::EBgp => "ebgp session",
                    BgpSessionType::IBgpPeer => "ibgp peer session",
                    BgpSessionType::IBgpClient => "ibgp client session",
                },
                src.index(),
                dst.index(),
            )
        })
        .join("\n");

    let bgp_propagations = n
        .get_known_prefixes()
        .map(|p| {
            format!(
                "    \\ifdefined\\prefix{}\n{}\n  \\fi",
                p.to_string().replace(['.', '/'], "_"),
                net.get_route_propagation(*p)
                    .into_iter()
                    .map(|(src, dst, _)| format!(
                        r"      \draw[bgp propagation] (r{}) to[bend left=20] (r{});",
                        src.index(),
                        dst.index(),
                    ))
                    .join("\n")
            )
        })
        .join("\n");

    LATEX_TEMLPATE
        .replace("{{PREFIXES}}", &prefix_choices)
        .replace("{{INTERNAL_NODES}}", &internal_nodes)
        .replace("{{EXTERNAL_NODES}}", &external_nodes)
        .replace("{{EDGES}}", &edges)
        .replace("{{NEXT_HOPS}}", &next_hops)
        .replace("{{LINK_WEIGHTS}}", &link_weights)
        .replace("{{BGP_SESSIONS}}", &bgp_sessions)
        .replace("{{BGP_PROPAGATIONS}}", &bgp_propagations)
}