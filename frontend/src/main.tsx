import React from 'react';
import ReactDOM from 'react-dom/client';
import {
  BrowserRouter, Route, Routes
} from "react-router-dom";
import App from './App';
import './index.css';
import Items from './Items';
import JoinGamePage from './JoinGamePage';
import Room from './Room';


ReactDOM.createRoot(document.getElementById('root')!).render(
  <BrowserRouter>
    <Routes>
      <Route path="/chat/:room" element={<Room />} />
      <Route path="/join" element={<JoinGamePage />} />
      <Route path="/items" element={<Items />} />
      <Route path="/" element={<App />} />
    </Routes>
  </BrowserRouter>
)
