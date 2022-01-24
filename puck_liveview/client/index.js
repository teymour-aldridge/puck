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
            console.log(document.getElementById(data.el));
            let split = data["payload"].split("+");
            console.log(split[1]);
            let el = document.getElementById(data.el);
            el.setAttribute(split[0], split[1]);
        } else if (data.ty == "removeAttr") {
            document.getElementById(data.el).removeAttribute(data.payload);
        } else if (data.ty === "setTagName") {
            let old_tag = document.getElementById(data.el);
            let new_tag = document.createElement(data.payload);
            new_tag.innerHTML = old_tag.innerHTML;
            for (var i = 0, l = old_tag.attributes.length; i < l; ++i) {
                let name = old_tag.attributes.item(i).nodeName;
                var value = old_tag.attributes.item(i).nodeValue;

                new_tag.setAttribute(name, value);
            }
            old_tag.parentNode.replaceChild(new_tag, old_tag);
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
        } else if (data.ty === "removeListeners") {
            let old = document.getElementById(data.el);
            let new_el = old.cloneNode(true);
            old.parentNode.replaceChild(new_el, old);
        } else if (data.ty === "deleteEl") {
            let el = document.getElementById(data.el);
            console.log("deleting");
            console.log(el);
            el.parentNode.removeChild(el);
        } else if (data.ty === "setId") {
            console.log("updating id, from: " + data.el + ", to: " + data.payload);
            document.getElementById(data.el).id = data.payload;
        }
    });
}
