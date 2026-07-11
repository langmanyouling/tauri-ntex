# 1. 初始化 Git 仓库

git init

New-Item .gitignore

# 2. 把所有文件添加到暂存区

git add .

# 3. 提交到本地仓库

git commit -m "initial commit"

# 4. 关联你的远程 GitHub/Gitee 仓库（把后面的网更换成你自己的仓库地址）

git remote add origin https://github.com/你的用户名/你的仓库名.git

# 5. 重命名主分支为 main（可选，但建议）

git branch -M main

# 6. 推送代码

git push -u origin main
