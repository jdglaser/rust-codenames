import React from 'react';
import ReactDOM from 'react-dom/client';
import {
  BrowserRouter, Route, Routes
} from "react-router-dom";
import App from './App';
import Chat from './Chat';
import './index.css';
import Items from './Items';
import JoinGamePage from './JoinGamePage';


ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/chat/:room" element={<Chat />} />
        <Route path="/join" element={<JoinGamePage />} />
        <Route path="/items" element={<Items />} />
        <Route path="/" element={<App />} />
      </Routes>
    </BrowserRouter>
  </React.StrictMode>
)
