import { useState } from 'react';
import { useNavigate } from "react-router-dom";
import "./App.css";

function App() {
  const [name, setName] = useState<string>("");

  const navigate = useNavigate();

  return (
    <div className="app">
        <div className="content">
          <h1>Codenames</h1>
          <div className='username-holder'>
            <label>Enter a game name</label>
            <input type="text"
                  onChange={(evt) => {
                      evt.preventDefault();
                      setName(evt.target.value)
                  }}
                  value={name} />
          </div>
          <div className='buttons'>
            <button onClick={() => navigate(`/chat/${name}`)} disabled={name === ""}>Join game</button>
          </div>
        </div>
    </div>
  )
}

export default App
