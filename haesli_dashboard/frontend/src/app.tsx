import React from 'react';
import DataPage from './components/data-page';
import './app.css';

const IS_PROD = process.env.NODE_ENV === 'production';

const URL_PREFIX = IS_PROD ? '' : 'http://localhost:8080/';

const App = () => {
  return (
    <div className="app">
      <header className="header">
        <h1>Haesli Dashboard</h1>
      </header>
      <DataPage prefix={URL_PREFIX} />
    </div>
  );
};

export default App;
