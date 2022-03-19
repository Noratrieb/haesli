import React, { FC } from 'react';

type Cell = string | number;

type Row = ReadonlyArray<Cell>;

type Props = {
  headers: ReadonlyArray<string>;
  rows: ReadonlyArray<Row>;
};

const Table: FC<Props> = ({ headers, rows }) => {
  return (
    <table>
      <tr>
        {headers.map((header) => (
          <th>{header}</th>
        ))}
      </tr>
      {rows.map((row) => (
        <tr>
          {row.map((cell) => (
            <td>{cell}</td>
          ))}
        </tr>
      ))}
    </table>
  );
};

export default Table;
