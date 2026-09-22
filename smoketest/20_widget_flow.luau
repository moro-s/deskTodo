--!strict
-- 验证 deskTodo 控件自动化：输入文本、读取状态、清空，不触发真实提交
local input = eguidev.widget("todo.input")
assert(input ~= nil, "todo.input should be visible")
input:type_text("eguidev 自动化测试", { clear = true })
local typed_state = eguidev.widget("todo.input"):state()
assert(typed_state ~= nil, "typed input state should exist")
assert(
    typed_state.value == "eguidev 自动化测试",
    "typed text should be in the input, got: " .. tostring(typed_state.value)
)
input:type_text("", { clear = true })
local cleared_state = eguidev.widget("todo.input"):state()
if cleared_state ~= nil then
    assert(
        cleared_state.value == "" or cleared_state.value == nil,
        "input should be cleared, got: " .. tostring(cleared_state.value)
    )
end
return "widget automation ok"
