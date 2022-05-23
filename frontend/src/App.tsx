import { useState } from 'react';
import { Link } from 'react-router-dom';

function App() {
  const [name, setName] = useState<string>("");

  return (
    <div className="App">
        <h1>Welcome</h1>
        <input type="text"
               onChange={(evt) => {
                  evt.preventDefault();
                  setName(evt.target.value)
               }}
               value={name} />
        <Link to="/items">Items</Link>
        <Link to="/chat/main" state={{userName: name}}>Chat</Link>
    </div>
  )
}

export default App
