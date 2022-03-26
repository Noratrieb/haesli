export type Channel = {
  id: string;
  number: number;
};

export type Connection = {
  id: string;
  peerAddr: string;
  channels: ReadonlyArray<Channel>;
};

export type Queue = {
  id: string;
  name: string;
  durable: boolean;
  messages: number;
};

export type Binding = {
  queue: string;
  routingKey: string;
};

export type Exchange = {
  name: string;
  durable: boolean;
  bindings: ReadonlyArray<Binding>;
};

export type Data = {
  connections: ReadonlyArray<Connection>;
  queues: ReadonlyArray<Queue>;
  exchanges: ReadonlyArray<Exchange>;
};
