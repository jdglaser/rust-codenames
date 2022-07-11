import { useEffect, useState } from 'react';
import { useCookies } from 'react-cookie';
import { useNavigate } from "react-router-dom";
import "./App.css";

function App() {
  const [name, setName] = useState<string>("");
  const [usernameText, setUsernameText] = useState<string>("");
  // const [usernameCookie, setUsernameCookie] = useState<string>("");

  const navigate = useNavigate();

  const [cookies, setCookie] = useCookies(['username']);

  const usernameIsSet = cookies.username !== undefined;
  const username = cookies.username;

  useEffect(() => {
    setUsernameText(usernameIsSet ? username : "");
  }, [cookies])

  return (
    <div style={{display: "flex",
                 flexDirection: "column",
                 justifyContent: "center",
                 alignItems: "center",
                 height: "100%",
                 padding: "20px"}}>
        <div>
          <h1 style={{textAlign: "center"}}>Codewords</h1>
          <h2 style={{textAlign: "center"}}>{usernameIsSet ? `Welcome Back ${username}` : "Hello New User"}!</h2>
          <div style={{display: "grid",
                       gridTemplateColumns: "auto auto",
                       gap: "10px",
                       justifyContent: "center"}}>
            <label>{usernameIsSet ? "Update Your Username" : "Set Your Username"}</label>
            <div />
            <input type="text"
              onChange={(evt) => {
                  evt.preventDefault();
                  setUsernameText(evt.target.value)
              }}
              value={usernameText} />
            <button disabled={usernameText == ""} onClick={() => setCookie("username", usernameText)}>{usernameIsSet ? "Update username" : "Set username"}</button>
            <label>Enter a game name</label>
            <div />
            <input type="text"
                  onChange={(evt) => {
                      evt.preventDefault();
                      setName(evt.target.value)
                  }}
                  value={name} />
            <button onClick={() => navigate(`/game/${name}`)} disabled={name === "" || !usernameIsSet}>Join game</button>
          </div>
        </div>
    </div>
  )
}

export default App
