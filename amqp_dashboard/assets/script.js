const renderTable = (colNames, rows) => {
  const table = document.createElement('table');

  const headerRow = document.createElement('tr');

  colNames.forEach((name) => {
    const th = document.createElement('th');
    th.innerText = name;

    headerRow.append(th);
  });
  table.append(headerRow);

  rows.forEach((row) => {
    const contentRow = document.createElement('tr');
    row.forEach((cell) => {
      const td = document.createElement('td');
      td.innerText = cell;
      contentRow.append(td);
    });
    table.append(contentRow);
  });

  return table;
};

const renderConnections = (connections) => {
  const wrapper = document.getElementById('connection-wrapper');

  const table = renderTable(
    ['Connection ID', 'Client Address', 'Channels'],
    connections.map((conn) => {
      const channels = conn.channels
        .map((chan) => `${chan.number} - ${chan.id}`)
        .join('\n');
      return [conn.id, conn.peer_addr, channels];
    })
  );
  wrapper.replaceChildren(table);
};

const refresh = async () => {
  const fetched = await fetch('http://localhost:3000/api/data');
  const data = await fetched.json();
  renderConnections(data.connections);
};

setInterval(refresh, 1000);
