import * as React from 'react';
import * as ReactDOM from 'react-dom';

import 'bootstrap/dist/css/bootstrap.min.css';

import { App } from './App';

const app = document.createElement("div");
app.id = "app";
document.body.appendChild(app);

ReactDOM.render(React.createElement(App), app)
