import React from 'react';
import { GraphView } from 'react-digraph';
import { Binding, Data, Exchange } from '../types';

const shape = (
  <symbol viewBox="0 0 100 100" id="empty" key="0">
    <circle cx="50" cy="50" r="30" />
  </symbol>
);

const graphConfig = {
  nodeTypes: {
    exchange: {
      // required to show empty nodes
      typeText: 'Exchange',
      shapeId: '#empty', // relates to the type property of a node
      shape,
    },
    queue: {
      // required to show empty nodes
      typeText: 'Queue',
      shapeId: '#empty', // relates to the type property of a node
      shape,
    },
    consumer: {
      // required to show empty nodes
      typeText: 'Consumer',
      shapeId: '#empty', // relates to the type property of a node
      shape,
    },
    channel: {
      // required to show empty nodes
      typeText: 'Channel',
      shapeId: '#empty', // relates to the type property of a node
      shape,
    },
    connection: {
      // required to show empty nodes
      typeText: 'Connection',
      shapeId: '#empty', // relates to the type property of a node
      shape,
    },
  },
  nodeSubtypes: {},
  edgeTypes: {},
};

type Props = {
  data: Data;
};

const SPACE_H = 120;
const SPACE_V = 150;

const EntityGraph = ({ data }: Props) => {
  const exchTotal = (data.exchanges.length * SPACE_H) / 2;
  const exchanges = data.exchanges.map((e, i) => ({
    id: e.name,
    title: e.name,
    y: 0,
    x: i * SPACE_H - exchTotal,
    type: 'exchange',
  }));

  const queueTotal = (data.queues.length * SPACE_H) / 2;
  const queues = data.queues.map((q, i) => ({
    id: q.name,
    title: q.name,
    y: SPACE_V,
    x: i * SPACE_H - queueTotal,
    type: 'queue',
  }));

  const consumersData = data.queues.flatMap((q) =>
    q.consumers.map((c) => [q, c] as const)
  );
  const consumerTotal = (consumersData.length * SPACE_H) / 2;
  const consumers = consumersData.map(([q, c], i) => ({
    id: c.tag,
    title: c.tag,
    y: SPACE_V * 2,
    x: i * SPACE_H - consumerTotal,
    type: 'consumer',
  }));

  const channelsData = data.connections.flatMap((c) => c.channels);
  const channelTotal = (channelsData.length * SPACE_H) / 2;
  const channels = channelsData.map((c, i) => ({
    id: c.id,
    title: c.number,
    y: SPACE_V * 3,
    x: i * SPACE_H - channelTotal,
    type: 'channel',
  }));

  const connectionTotal = (data.connections.length * SPACE_H) / 2;
  const connections = data.connections.map((c, i) => ({
    id: c.id,
    title: c.peerAddr,
    y: SPACE_V * 4,
    x: i * SPACE_H - connectionTotal,
    type: 'connection',
  }));

  const nodes = [
    ...queues,
    ...exchanges,
    ...consumers,
    ...channels,
    ...connections,
  ];

  const bindingEdges = data.exchanges
    .flatMap((e) => e.bindings.map((b) => [b, e] as const))
    .map(([b, e]) => ({
      source: b.queue,
      target: e.name,
      label_to: `'${b.routingKey}'`,
      type: 'emptyEdge',
    }));

  const consumerEdges = consumersData.map(([q, c]) => ({
    source: c.tag,
    target: q.name,
    type: 'emptyEdge',
  }));

  const channelConsumerEdges = consumersData.map(([, c]) => ({
    source: c.channel,
    target: c.tag,
    type: 'emptyEdge',
  }));

  const connectionChannelEdges = data.connections.flatMap((c) =>
    c.channels.map((ch) => ({
      source: c.id,
      target: ch.id,
      type: 'emptyEdge',
    }))
  );

  const edges = [
    ...bindingEdges,
    ...consumerEdges,
    ...channelConsumerEdges,
    ...connectionChannelEdges,
  ];

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
