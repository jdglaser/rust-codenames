import React, { useState } from "react";
import { useNavigate } from "react-router-dom";

export default function JoinGamePage() {
  const [gameName, setGameName] = useState<string>("");

  const navigate = useNavigate();

  return (
    <>
      <input type="text"
             onChange={(evt) => {
                evt.preventDefault();
                setGameName(evt.target.value)
             }}
             value={gameName} />
      <button disabled={gameName === ""} onClick={() => navigate(`/${gameName}`)}>Join</button>
    </>
  )
}