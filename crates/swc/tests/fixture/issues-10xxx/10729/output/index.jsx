var _require = require("react/jsx-runtime"), _jsx = _require.jsx;
var type = function() {
    return null;
};
var widget = {
    component: {
        type: type
    }
};
var render = function() {
    return /*#__PURE__*/ _jsx(widget.component.type, {
        if: "aaa"
    });
};
/*#__PURE__*/ _jsx(widget.component.type, {
    if: true,
    for: "loop",
    while: "waiting",
    null: null
});
/*#__PURE__*/ _jsx(widget.component.type, {
    if: true,
    null: null,
    for: "loop",
    while: "waiting"
});
