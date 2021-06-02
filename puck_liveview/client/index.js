let ws = new WebSocket("ws://" + window.location.host + window.location.pathname + "ws");

ws.onmessage = (msg) => {
    let data = JSON.parse(msg.data);
    data.map((data) => {
        if (data.ty === "createTag") {
            console.log("create tag");
            let split = data["payload"].split("+");
            let el = document.createElement(split[0]);
            el.id = data["el"];
            if (!split[1] || split[1] === "") {
                document.body.appendChild(el);
            } else {
                document.getElementById(split[1]).appendChild(el);
            }
        } else if (data.ty === "setAttr") {
            console.log("set attribute");
            let split = data["payload"].split("+");
            let el = document.getElementById(data["el"]);
            el.setAttribute(split[0], split[1]);
        } else if (data.ty === "setText") {
            console.log(data);
            document.getElementById(data["el"]).textContent = data["payload"];
        } else if (data.ty === "attachListener") {
            let split = data.payload.split("+");
            document.getElementById(data.el).addEventListener(split[1], function (e) {
                console.log("sending listener data");
                console.log(split);
                console.log(data);
                if (split[1] === "click") {
                    console.log("sending click data");
                    ws.send(JSON.stringify({
                        listener: split[0]
                    }));
                } else if (split[1] === "submit") {
                    ws.send(JSON.stringify({
                        listener: split[0]
                    }));
                } else if (split[1] === "input") {
                    console.log("sending input data");
                    ws.send(JSON.stringify({
                        listener: split[0],
                        payload: {
                            value: e.target.value
                        }
                    }));
                }
            })
        }
    });
}
