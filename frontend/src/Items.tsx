import axios from "axios";
import React, { useEffect, useState } from "react";

export default function Items() {
  const [items, setItems] = useState<any>([]);

  useEffect(() => {
    axios({
      method: 'get',
      url: '/api/items'
    })
    .then(function (response) {
      console.log("RES: ", response);
      setItems(response.data);
    });
  }, [])

  return (
    <>
      Items:
      {items.map((i: any) => i.name).join(",")}
    </>
  )
}