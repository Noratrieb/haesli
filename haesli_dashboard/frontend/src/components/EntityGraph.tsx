import React from 'react';
import { GraphView } from 'react-digraph';
import { Binding, Data, Exchange } from '../types';

const sample = {
  nodes: [
    {
      id: 1,
      title: 'Exchange A',
      type: 'empty',
    },
    {
      id: 2,
      title: 'Queue A',
      type: 'empty',
    },
  ],
  edges: [
    {
      source: 1,
      target: 2,
      type: 'emptyEdge',
    },
    {
      source: 2,
      target: 4,
      type: 'emptyEdge',
    },
  ],
};

const graphConfig = {
  nodeTypes: {
    exchange: {
      // required to show empty nodes
      typeText: 'Exchange',
      shapeId: '#empty', // relates to the type property of a node
      shape: (
        <symbol viewBox="0 0 100 100" id="empty" key="0">
          <circle cx="50" cy="50" r="45" />
        </symbol>
      ),
    },
    queue: {
      // required to show empty nodes
      typeText: 'Queue',
      shapeId: '#empty', // relates to the type property of a node
      shape: (
        <symbol viewBox="0 0 100 100" id="empty" key="0">
          <circle cx="50" cy="50" r="45" />
        </symbol>
      ),
    },
  },
  nodeSubtypes: {},
  edgeTypes: {},
};

type Props = {
  data: Data;
};

const SPACE = 200;

const EntityGraph = ({ data }: Props) => {
  const queueTotal = (data.queues.length * SPACE) / 2;
  const queues = data.queues.map((q, i) => ({
    id: q.name,
    title: q.name,
    y: 300,
    x: i * SPACE - queueTotal,
    type: 'queue',
  }));
  const exchTotal = (data.exchanges.length * SPACE) / 2;
  const exchanges = data.exchanges.map((e, i) => ({
    id: e.name,
    title: e.name,
    y: 0,
    x: i * SPACE - exchTotal,
    type: 'exchange',
  }));

  const nodes = [...queues, ...exchanges];

  const edges = data.exchanges
    .flatMap((e) => e.bindings.map((b) => [b, e] as [Binding, Exchange]))
    .map(([b, e]) => ({
      source: b.queue,
      target: e.name,
      label_to: `'${b.routingKey}'`,
      type: 'emptyEdge',
    }));

  const nodeTypes = graphConfig.nodeTypes;
  const nodeSubtypes = graphConfig.nodeSubtypes;
  const edgeTypes = graphConfig.edgeTypes;

  return (
    <div className="graph">
      <GraphView
        nodeKey="id"
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        nodeSubtypes={nodeSubtypes}
        edgeTypes={edgeTypes}
      />
    </div>
  );
};

export default EntityGraph;
