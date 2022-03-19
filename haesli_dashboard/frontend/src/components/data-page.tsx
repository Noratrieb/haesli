import React, { FC, useCallback, useEffect, useState } from 'react';
import Table from './table';
import type { Data } from '../types';

const fetchData = async (prefix: string): Promise<Data> => {
  const url = `${prefix}api/data`;
  return fetch(url).then((res) => res.json());
};

type Props = {
  prefix: string;
};

const DataPage: FC<Props> = ({ prefix }) => {
  const [data, setData] = useState<Data | null>(null);

  const refresh = useCallback(async () => {
    const newData = await fetchData(prefix);
    setData(newData);
  }, [setData, prefix]);

  useEffect(() => {
    const interval = setInterval(refresh, 1000);

    return () => clearInterval(interval);
  }, [refresh]);

  return (
    <div>
      <section>
        <h2>Connections</h2>
        {data ? (
          <Table
            headers={['Connection ID', 'Client Address', 'Channels']}
            rows={data.connections.map((connection) => [
              connection.id,
              connection.peerAddr,
              connection.channels.length,
            ])}
          />
        ) : (
          <div>Loading...</div>
        )}
      </section>
      <section>
        <h2>Queues</h2>
        {data ? (
          <Table
            headers={['Queue ID', 'Name', 'Durable', 'Message Count']}
            rows={data.queues.map((queue) => [
              queue.id,
              queue.name,
              queue.durable ? 'Yes' : 'No',
              queue.messages,
            ])}
          />
        ) : (
          <div>Loading...</div>
        )}
      </section>
      <section>
        <h2>Channels</h2>
        {data ? (
          <Table
            headers={['Channel ID', 'Connection ID', 'Number']}
            rows={data.connections
              .map((connection) =>
                connection.channels.map((channel) => ({
                  ...channel,
                  connectionId: connection.id,
                }))
              )
              .flat()
              .map((channel) => [
                channel.id,
                channel.connectionId,
                channel.number,
              ])}
          />
        ) : (
          <div>Loading...</div>
        )}
      </section>
    </div>
  );
};

export default DataPage;
