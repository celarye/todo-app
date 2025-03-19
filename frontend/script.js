const API = "http://localhost:8080";
const gitHubLoginBtn = document.querySelector('.github-login-btn');
const logoutBtn = document.querySelector('.logout-btn');
const userInfo = document.querySelector('.user-info');
const usernameElement = document.querySelector('.username');
const emailElement = document.querySelector('.email');
const profileImgElement = document.querySelector('.profile-img');
const loginPrompt = document.querySelector('.login-prompt');
const todoContainer = document.querySelector('.todo-container');
const userCountElement = document.querySelector('.user-count span');

function checkIfLoggedIn() {
	let cookies = document.cookie.split(";");
	for (let cookie of cookies) {
		const [name, value] = cookie.trim().split('=');
		if (name === 'loggedin') {
			console.log('Session cookie found. User is logged in.');
			return value;
		}
	}
	console.log('No session cookie found. User is not logged in.');
}

function updateUserCount() {
	fetch(`${API}/`, {
		method: 'GET',
		credentials: 'include'
	})
		.then(response => {
			if (response.ok) {
				return response.json();
			}
			throw new Error('Failed to fetch user count');
		})
		.then(data => {
			userCountElement.textContent = data.user_count;
			console.log(`User count updated: ${data.user_count}`);
		})
		.catch(error => {
			console.error('Error fetching user count:', error);
		});
}

function renderTodoItems(items) {
	const todoListElement = document.querySelector('.todo-list');
	todoListElement.innerHTML = '';
	items.forEach(item => {
		const itemElement = document.createElement('div');
		itemElement.classList.add('todo-item');
		itemElement.innerHTML = `
            <input type="checkbox" ${item.done ? 'checked' : ''} onchange="updateTodoItem(${item.id}, this.checked)">
            <span>${item.content}</span>
            <button onclick="deleteTodoItem(${item.id})">Delete</button>
        `;
		todoListElement.appendChild(itemElement);
	});
	console.log('TODO items rendered.');
}

function getTodoItems() {
	fetch(`${API}/todo/`, {
		method: 'GET',
		credentials: 'include'
	})
		.then(response => {
			if (response.ok) {
				return response.json();
			}
			throw new Error('Failed to fetch TODO items');
		})
		.then(items => {
			renderTodoItems(items);
		})
		.catch(error => {
			console.error('Error fetching TODO items:', error);
		});
}

function addTodoItem(content) {
	fetch(`${API}/todo/set`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ content }),
		credentials: 'include'
	})
		.then(response => {
			if (response.ok) {
				console.log(`New TODO item added: "${content}"`);
				getTodoItems();
			} else {
				throw new Error('Failed to add TODO item');
			}
		})
		.catch(error => {
			console.error('Error adding TODO item:', error);
		});
}

function updateTodoItem(itemId, done) {
	fetch(`${API}/todo/update/${itemId}`, {
		method: 'PATCH',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ done }),
		credentials: 'include'
	})
		.then(response => {
			if (response.ok) {
				console.log(`TODO item updated (ID: ${itemId}, Done: ${done})`);
				getTodoItems();
			} else {
				throw new Error('Failed to update TODO item');
			}
		})
		.catch(error => {
			console.error('Error updating TODO item:', error);
		});
}

function deleteTodoItem(itemId) {
	fetch(`${API}/todo/delete/${itemId}`, {
		method: 'DELETE',
		credentials: 'include'
	})
		.then(response => {
			if (response.ok) {
				console.log(`TODO item deleted (ID: ${itemId})`);
				getTodoItems();
			} else {
				throw new Error('Failed to delete TODO item');
			}
		})
		.catch(error => {
			console.error('Error deleting TODO item:', error);
		});
}

document.addEventListener('DOMContentLoaded', function() {
	const logged_in = checkIfLoggedIn();
	const urlParams = new URLSearchParams(window.location.search);
	const code = urlParams.get('code');
	const state = urlParams.get('state');
	const loginPrompt = document.querySelector('.login-prompt');

	if (logged_in) {
		todoContainer.style.display = 'block';
		loginPrompt.style.display = 'none';

		fetch(`${API}/user/`, {
			method: 'GET',
			credentials: 'include'
		})
			.then(response => {
				if (response.ok) {
					return response.json();
				}
				throw new Error('Failed to fetch user info');
			})
			.then(user => {
				userInfo.classList.add('show');
				gitHubLoginBtn.style.display = 'none';
				usernameElement.textContent = user.username;
				emailElement.textContent = user.email;
				if (user.profile_picture) {
					profileImgElement.src = user.profile_picture_url;
				}
				console.log(`User info fetched for ${user.username}`);
			})
			.catch(error => {
				console.error('Error fetching user info:', error);
			});

		logoutBtn.addEventListener('click', function() {
			document.cookie = 'sessionid=; Max-Age=0; Path=/; Domain=todo.celarye.dev;';
			document.cookie = 'loggedin=; Max-Age=0; Path=/; Domain=todo.celarye.dev;';
			fetch(`${API}/user/logout`, {
				method: 'DELETE',
				credentials: 'include'
			}).catch();

			console.log('User logged out.');
			window.location.reload();
		});

		getTodoItems();

		document.querySelector('.add-todo-btn').addEventListener('click', () => {
			const todoContent = document.querySelector('.new-todo-input').value;
			if (todoContent) {
				addTodoItem(todoContent);
				document.querySelector('.todo-input').value = '';
			} else {
				console.log('No content entered for new TODO item.');
			}
		});
	} else if (code && state) {
		console.log('GitHub redirect detected, exchanging code for session...');
		fetch(`${API}/user/auth/github/success`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ code, csrf_token: state }),
			credentials: 'include'
		})
			.then(response => {
				if (!response.ok) {
					throw new Error('Failed to complete GitHub login');
				}
				document.cookie = 'loggedin=true; Max-Age=21540; Path=/; Secure; Partitioned; Domain=todo.celarye.dev;';
				if (history.pushState) {
					const newUrl = window.location.origin + window.location.pathname;
					history.replaceState(null, '', newUrl);
				}

				console.log('GitHub login successful.');
				gitHubLoginBtn.style.display = 'none';

				todoContainer.style.display = 'block';
				loginPrompt.style.display = 'none';

				fetch(`${API}/user/`, {
					method: 'GET',
					credentials: 'include'
				})
					.then(response => {
						if (response.ok) {
							return response.json();
						}
						throw new Error('Failed to fetch user info');
					})
					.then(user => {
						userInfo.classList.add('show');
						usernameElement.textContent = user.username;
						emailElement.textContent = user.email;
						profileImgElement.src = user.profile_picture_url;
						updateUserCount();
					})
					.catch(error => {
						console.error('Error fetching user info:', error);
					});

				getTodoItems();

				logoutBtn.addEventListener('click', function() {
					document.cookie = 'sessionid=; Max-Age=0; Path=/; Domain=todo.celarye.dev;';
					document.cookie = 'loggedin=; Max-Age=0; Path=/; Domain=todo.celarye.dev;';
					fetch(`${API}/user/logout`, {
						method: 'DELETE',
						credentials: 'include'
					}).catch();

					console.log('User logged out.');
					window.location.reload();
				});

				document.querySelector('.add-todo-btn').addEventListener('click', () => {
					const todoContent = document.querySelector('.new-todo-input').value;
					if (todoContent) {
						addTodoItem(todoContent);
					}
				});
			})
			.catch(error => {
				console.error('Error during GitHub login process:', error);
			});
	} else {
		gitHubLoginBtn.addEventListener('click', async function() {
			try {
				const response = await fetch(`${API}/user/auth/github/init`);
				if (response.ok) {
					const init = await response.json();
					window.location.href = init.redirect_url;
				} else {
					throw new Error('Failed to initiate GitHub login');
				}
			} catch (error) {
				console.error('Error initiating GitHub login:', error);
			}
		});
	}

	updateUserCount();
});

