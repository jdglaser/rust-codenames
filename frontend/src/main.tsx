import React from 'react';
import { CookiesProvider } from 'react-cookie';
import ReactDOM from 'react-dom/client';
import {
  BrowserRouter, Route, Routes
} from "react-router-dom";
import App from './App';
import './index.css';
import Room from './Room';


ReactDOM.createRoot(document.getElementById('root')!).render(
  <CookiesProvider>
    <BrowserRouter>
      <div style={{height: "100vh", maxHeight: "100vh"}}>
        <Routes>
          <Route path="/game/:room" element={<Room />} />
          <Route path="/" element={<App />} />
        </Routes>
      </div>
    </BrowserRouter>
  </CookiesProvider>
)
