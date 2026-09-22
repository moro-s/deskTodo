--!strict
-- 验证「清除已完成」按钮永远悬浮显示（空列表日期也存在）
local prev = eguidev.widget("titlebar.prev_month")
assert(prev ~= nil, "titlebar.prev_month should be visible")
prev:click()
local clear_button = eguidev.widget("todo.clear_done")
assert(clear_button ~= nil, "clear button should exist even on an empty day")
local next = eguidev.widget("titlebar.next_month")
assert(next ~= nil, "titlebar.next_month should be visible")
next:click()
return "clear button always visible"
