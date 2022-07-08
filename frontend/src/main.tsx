import React from 'react';
import { CookiesProvider } from 'react-cookie';
import Div100vh from 'react-div-100vh';
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
      <Div100vh>
        <Routes>
          <Route path="/game/:room" element={<Room />} />
          <Route path="/" element={<App />} />
        </Routes>
      </Div100vh>
    </BrowserRouter>
  </CookiesProvider>
)
