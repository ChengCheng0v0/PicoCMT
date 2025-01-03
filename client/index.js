/* global SparkMD5*/

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
function getGravatarUrl(email, id) {
    if (email) {
        const hash = SparkMD5.hash(email.trim().toLowerCase());
        const gravatarUrl = `https://www.gravatar.com/avatar/${hash}?d=404`;

        try {
            const response = fetch(gravatarUrl);
            if (response.ok) {
                return `https://www.gravatar.com/avatar/${hash}`;
            }
        } catch (e) {
            console.error("从 Gravatar 获取头像时出现错误: ", e);
        }

        return `https://www.loliapi.com/acg/pp/?random=${email}`;
    }

    return `https://www.loliapi.com/acg/pp/?random=${id}`;
}

// 创建评论元素
function createCommentElement(comment, alignment) {
    const { id, parent_id, nickname, email, content } = comment;
    const shortId = `#${id.slice(0, 4)}`;
    const replyId = parent_id ? `, Re: #${parent_id.slice(0, 4)}` : "";

    const commentDiv = document.createElement("div");
    commentDiv.className = `comment ${alignment}`;

    commentDiv.innerHTML = `
        <div class="avatar">
            <img src="${getGravatarUrl(email, id)}" alt="" />
        </div>
        <div class="bubble">
            <div class="meta">
                <span class="id">${shortId}${replyId}</span>
                <span class="nickname">${nickname}</span>
                ${email ? `<span class="email">${email}</span>` : ""}
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
        const commentElement = createCommentElement(comment, alignment);
        container.appendChild(commentElement);

        // 获取子评论并递归渲染
        const subComments = await fetchComments(`http://127.0.0.1:3000/api/get_sub_comments?parent_id=${comment.id}`);
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

    const topComments = await fetchComments("http://127.0.0.1:3000/api/get_top_comments");
    await renderComments(topComments, commentsContainer, "left");
}

document.addEventListener("DOMContentLoaded", () => {
    // 注入 PicoCMT 的 HTML 元素
    document.getElementById("picocmt-inject").innerHTML = `
        <div class="send">
            <div class="title"><i class="fa-solid fa-pen-to-square"></i><span>撰写评论</span></div>
            <textarea id="comment-content" class="editor" placeholder="编辑评论内容..." maxlength="256" required></textarea>
            <div class="bottom">
                <input class="nickname" type="text" placeholder="昵称" />
                <input class="email" type="email" placeholder="邮箱" />
                <button class="send-button"><i class="fa-solid fa-paper-plane"></i>发送</button>
            </div>
        </div>
        <div class="comments"></div>
    `;

    // 启动评论渲染
    initComments().catch(err => console.error("无法初始化评论: ", err));
});
