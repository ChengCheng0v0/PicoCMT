/* global SparkMD5*/

import "https://lf6-cdn-tos.bytecdntp.com/cdn/expire-1-M/spark-md5/3.0.2/spark-md5.min.js";

var picocmt = {};

// 控制通知
function notify(operation, type, title, content) {
    const notifyBody = document.querySelector(".picocmt > .notify");
    const notifyTitle = document.querySelector(".picocmt > .notify > .title");
    const notifyContent = document.querySelector(".picocmt > .notify > .content");

    switch (operation) {
        case "open":
            // 关闭先前通知
            notify("close");
            // 打开新通知
            notifyBody.classList.value = `notify ${type}`;
            notifyTitle.innerText = title;
            notifyContent.innerText = content;

            break;

        case "close":
            // 关闭通知
            notifyBody.classList.value = "notify";
            notifyTitle.innerText = "";
            notifyContent.innerText = "";

            break;

        default:
            break;
    }
}

// 获取评论数据
async function fetchComments(url) {
    const response = await fetch(url);
    if (!response.ok) {
        console.error("无法获取评论数据: ", response.statusText);
        return [];
    }
    return response.json();
}

// 获取 Gravatar 头像
async function getGravatarUrl(email, nickname) {
    if (email) {
        const emailHash = SparkMD5.hash(email.trim().toLowerCase());
        const gravatarUrl = `https://www.gravatar.com/avatar/${emailHash}?d=404`;

        try {
            const response = await fetch(gravatarUrl);
            if (response.ok) {
                return `https://www.gravatar.com/avatar/${emailHash}`;
            }
        } catch (e) {
            console.error("从 Gravatar 获取头像时出现错误: ", e);
        }

        return `https://www.loliapi.com/acg/pp/?random=${emailHash.slice(0, 8)}`;
    }

    const nicknameHash = SparkMD5.hash(nickname.trim().toLowerCase());
    return `https://www.loliapi.com/acg/pp/?random=${nicknameHash}`;
}

// 创建评论元素
async function createCommentElement(comment, alignment) {
    const { id, parent_id, nickname, email, content } = comment;
    const shortId = `#${id.slice(0, 4)}`;
    const replyId = parent_id ? ` <i class="fa-solid fa-reply" style="transform: rotateY(180deg);"></i> #${parent_id.slice(0, 4)}` : "";

    const avatarUrl = await getGravatarUrl(email, nickname);

    const commentDiv = document.createElement("div");
    commentDiv.className = `comment ${alignment}`;

    commentDiv.innerHTML = `
        <div class="avatar">
            <img src="${avatarUrl}" alt="" />
        </div>
        <div class="bubble">
            <div class="meta">
                <span class="id"><i class="fa-solid fa-circle"></i> ${shortId}${replyId}</span>
                <span class="nickname"><i class="fa-solid fa-user-pen"></i> ${nickname}</span>
                ${email ? `<span class="email"><i class="fa-solid fa-envelope"></i> ${email}</span>` : ""}
            </div>
            <div class="content">
                <span>${content}</span>
            </div>
        </div>
    `;

    return commentDiv;
}

// 递归渲染评论
async function renderComments(comments, container, alignment) {
    for (const comment of comments) {
        const commentElement = await createCommentElement(comment, alignment);
        container.appendChild(commentElement);

        // 获取子评论并递归渲染
        const subComments = await fetchComments(`${picocmt.config.server}/api/get_sub_comments?parent_id=${comment.id}`);
        if (subComments.length > 0) {
            await renderComments(subComments, container, "right");
        }
    }
}

// 初始化评论渲染
async function initComments() {
    const commentsContainer = document.querySelector(".picocmt > .comments");
    if (!commentsContainer) {
        console.error("找不到 .comments 元素");
        return;
    }

    const topComments = await fetchComments(`${picocmt.config.server}/api/get_top_comments`);
    await renderComments(topComments, commentsContainer, "left");
}

// 发送评论
async function sendComment() {
    // 获取需要发送的内容
    let comment = {
        nickname: document.querySelector(".picocmt > .send > .bottom > .nickname").value,
        email: document.querySelector(".picocmt > .send > .bottom > .email").value,
        content: document.querySelector(".picocmt > .send > .editor").value,
    };

    // 检查输入是否合法
    if (!comment.content) {
        notify("open", "warn", "评论无法发送", "评论内容不能为空。");
        return;
    } else if (comment.content.length > 256) {
        notify("open", "warn", "评论无法发送", "评论内容长度不能超过 256 字符，请检查并修改评论内容。");
        return;
    }
    if (!comment.nickname) {
        notify("open", "warn", "评论无法发送", "昵称不能为空，请输入一个长度不超过 16 字符的昵称。");
        return;
    } else if (comment.nickname.length > 16) {
        notify("open", "warn", "评论无法发送", "昵称长度不能超过 16 字符，请检查并修改昵称设置。");
        return;
    }
    if (comment.email.length > 32) {
        notify("open", "warn", "评论无法发送", "邮箱长度不能超过 32 字符，请检查并修改邮箱设置。");
        return;
    } else if (comment.email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(comment.email)) {
        notify("open", "warn", "评论无法发送", "邮箱格式不合法，请检查并修改邮箱设置。");
        return;
    }

    // 构建请求
    const apiUrl = `${picocmt.config.server}/api/add_comment`;
    const sendOptions = {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: `{"parent_id":null,"nickname":"${comment.nickname}","email":"${comment.email ? comment.email : ""}","content":"${comment.content}"}`,
    };

    // 发送请求
    try {
        const response = await fetch(apiUrl, sendOptions);
        if (response.ok) {
            const data = await response.json();
            notify("open", "info", "评论发送成功！", "o(*≧▽≦)ツ┏━┓");

            // 补全评论数据
            comment.id = data.comment.id;
            comment.create_at = data.comment.create_at;

            // 渲染刚发送的评论
            const commentElement = await createCommentElement(comment, "left");
            document.querySelector(".picocmt > .comments").prepend(commentElement);
        } else {
            notify("open", "error", `评论发送失败: ${response.status} (${response.statusText})`, await response.text());
        }
    } catch (e) {
        notify("open", "error", "评论发送失败", e);
    }
}

document.addEventListener("DOMContentLoaded", () => {
    picocmt.config = document.getElementById("picocmt-script").dataset;

    setTimeout(() => {
        picocmt.element = document.getElementById("picocmt-inject");

        // 注入 PicoCMT 的 HTML 元素
        picocmt.element.innerHTML = `
            <div class="notify">
                <div class="title"></div>
                <button class="close"><i class="fa-solid fa-circle-xmark"></i></button>
                <div class="content"></div>
            </div>
            <div class="send">
                <div class="title"><i class="fa-solid fa-pen-to-square"></i><span>撰写评论</span></div>
                <textarea class="editor" placeholder="编辑评论内容..." maxlength="256" required></textarea>
                <div class="bottom">
                    <input class="nickname" type="text" placeholder="昵称" required />
                    <input class="email" type="email" placeholder="邮箱 (选填)" />
                    <button class="send-button"><i class="fa-solid fa-paper-plane"></i>发送</button>
                </div>
            </div>
            <div class="comments"></div>
        `;

        // 启动评论渲染
        initComments().catch(err => console.error("无法初始化评论: ", err));

        // 添加关闭通知的操作监听
        document.querySelector(".picocmt > .notify > .close").addEventListener("click", () => {
            notify("close");
        });

        // 添加发送评论的操作监听
        document.querySelector(".picocmt > .send > .bottom > .send-button").addEventListener("click", () => {
            sendComment();
        });
    }, picocmt.config.load_delay);
});
