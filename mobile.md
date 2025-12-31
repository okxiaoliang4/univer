现在univer的 @packages/sheets-ui/ 对mobile端支持的不是很好，我需要做一套新的符合移动端操作的UI
1. 当前聚焦到单元格的时候直接就弹起了键盘，我希望是聚焦到单元格的时候右下角浮动一个按钮，参考 @packages-experimental/debugger/src/views/Fab.tsx ，里面的图标是键盘，点击之后才弹起来自定义的 keyboard 输入模式.
输入模式打开的时候根据 cell.t 的type类型来决定显示哪个输入模式

这个keyboard布局是这样的：
最上面是几个按钮操作，左到右依次为
1. Undo，撤销
2. Redo，重做
3. Copy，复制
4. Paste，粘贴
5. Cut，剪切
6. Clear，清除

在这块区域的下面是 @packages/sheets-ui/src/views/formula-bar/FormulaBar.tsx 里面的 FormulaEditor 组件，最右边是一个勾按钮，点击之后结束编辑，隐藏keyboard。
在这块区域的下面是几个快捷按钮，切换输入模式，左到右依次为
1. "Tab"，点击之后确认当前输入内容，并将selection切换到右边的cell
2. "Formula"，选中时下面的内容为函数键盘
3. "123"，选中时下面的内容为数字键盘
4. "ABC"，选中时下面的内容为原生键盘

## 当选中formula键盘时布局如下：

`Undo` `Redo` `Copy` `Paste` `Cut` `Clear`

`FormularBar`

`Tab` `f(x)` `123` `ABC` `↵`

|1|2|3|4|5|6|7|8|9|0|
|-|-|-|-|-|-|-|-|-|-|
|+|-|×|÷|||||f(x)|del|
|+|(|)|,|||||Σ|tab|
|<|>|:|.|||||""|↵|
|$|%|&|^|||||space|A1

---
+-×÷ 这块区域需要比数字的按键大一点并且需要与f(x)功能区进行间隔

当点击f(x)时弹起一个panel，里面需要列出来所有的 formula，参考 @packages/sheets-formula/src/services/function-list/function-list.ts
@packages/sheets-formula/src/services/register-function.service.ts
@packages/sheets-formula/src/services/description.service.ts
进行选择函数，当点击函数之后插入函数内容至FormulaBar中然后光标移动到新插入的函数的括号中，例如点击了SUM,则插入=SUM()，光标在括号中。

这里还有个Σ,点击也是插入SUM函数。

点击 "" 时插入双引号，然后光标移动在双引号中间。
点击↵时即确认保存当前单元格内容，然后移动选区到同列下一行的单元格
点击space时插入空格
点击A1时切换到自定义的英文输入键盘，将数字行下面的内容替换为
q w e r t y u i o p
 a s d f g h j k l
⇧ z x c v b n m del
$ : , ! space ↵ back

点击back之后返回到formula输入模式

## 当选中数字键盘时布局如下

`Undo` `Redo` `Copy` `Paste` `Cut` `Clear`

`FormularBar`

`Tab` `f(x)` `123` `ABC` `↵`

| | | | | |
|-|-|-|-|-|
|+/-|7|8|9|del|
|%|4|5|6|tab|
|$|1|2|3|↵|
|¥|00|0|.|↵|

当选中ABC时直接弹起来原生键盘进行输入