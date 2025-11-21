<script lang="ts">
  import { onMount } from 'svelte';
  import { chat, listModels, saveChatToLocal, loadChatHistory, type Message, type ModelInfo, type ChatResponse } from '../lib/api/mcp';

  let messages: Message[] = [];
  let inputMessage = '';
  let selectedModel = 'llama3:8b-instruct-q4';
  let models: ModelInfo[] = [];
  let isLoading = false;
  let error = '';
  let lastResponse: ChatResponse | null = null;

  // Quick actions
  const quickActions = [
    { label: 'Résumer PV', prompt: 'Résume le dernier procès-verbal de l\'assemblée générale en 3 points clés.' },
    { label: 'Traduire FR→EN', prompt: 'Traduis le texte suivant en anglais:' },
    { label: 'OCR Facture', prompt: 'Extrais les informations de cette facture (fournisseur, montant, date, numéro):' },
    { label: 'Calculer Charges', prompt: 'Calcule la répartition des charges communes pour cette copropriété selon les quotes-parts:' },
  ];

  onMount(async () => {
    // Load available models
    try {
      models = await listModels();
    } catch (e) {
      console.error('Failed to load models:', e);
      error = 'Impossible de charger les modèles. Vérifiez que le serveur MCP est démarré.';
    }

    // Load chat history from IndexedDB
    try {
      const history = await loadChatHistory();
      if (history.length > 0) {
        const last = history[history.length - 1];
        messages = last.messages || [];
      }
    } catch (e) {
      console.error('Failed to load history:', e);
    }
  });

  async function sendMessage() {
    if (!inputMessage.trim() || isLoading) return;

    error = '';
    const userMessage: Message = {
      role: 'user',
      content: inputMessage.trim(),
    };

    messages = [...messages, userMessage];
    inputMessage = '';
    isLoading = true;

    try {
      const response = await chat({
        model: selectedModel,
        messages,
        context: 'copro:demo',
        temperature: 0.7,
      });

      const assistantMessage: Message = {
        role: 'assistant',
        content: response.content,
      };

      messages = [...messages, assistantMessage];
      lastResponse = response;

      // Save to IndexedDB
      await saveChatToLocal(messages, response, 'copro:demo');
    } catch (e: any) {
      error = e.message || 'Erreur lors de l\'envoi du message';
      console.error('Chat error:', e);
    } finally {
      isLoading = false;
    }
  }

  function useQuickAction(action: typeof quickActions[0]) {
    inputMessage = action.prompt;
  }

  function clearChat() {
    messages = [];
    lastResponse = null;
    error = '';
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  }
</script>

<div class="mcp-chatbot">
  <div class="chatbot-header">
    <h2>🤖 Assistant IA KoproGo</h2>
    <div class="header-controls">
      <select bind:value={selectedModel} class="model-select">
        {#each models as model}
          <option value={model.id}>
            {model.name} {model.edge_compatible ? '🍓' : '☁️'}
          </option>
        {/each}
      </select>
      <button on:click={clearChat} class="btn-clear">🗑️ Effacer</button>
    </div>
  </div>

  {#if error}
    <div class="error-banner">
      ⚠️ {error}
    </div>
  {/if}

  <div class="quick-actions">
    {#each quickActions as action}
      <button
        on:click={() => useQuickAction(action)}
        class="quick-action-btn"
        disabled={isLoading}
      >
        {action.label}
      </button>
    {/each}
  </div>

  <div class="messages-container">
    {#if messages.length === 0}
      <div class="empty-state">
        <p>💬 Posez une question à l'assistant IA</p>
        <p class="text-muted">Modèle local (0g CO₂) ou cloud selon vos besoins</p>
      </div>
    {/if}

    {#each messages as message, i}
      <div class="message message-{message.role}">
        <div class="message-avatar">
          {#if message.role === 'user'}
            👤
          {:else if message.role === 'assistant'}
            🤖
          {:else}
            ⚙️
          {/if}
        </div>
        <div class="message-content">
          <div class="message-text">{message.content}</div>
          {#if message.role === 'assistant' && i === messages.length - 1 && lastResponse}
            <div class="message-meta">
              <span class="meta-item">
                {lastResponse.execution_info.execution_type === 'edge' ? '🍓 Edge' : '☁️ Cloud'}
              </span>
              <span class="meta-item">⏱️ {lastResponse.execution_info.latency_ms}ms</span>
              <span class="meta-item">🎫 {lastResponse.usage.total_tokens} tokens</span>
              <span class="meta-item">
                🌱 {lastResponse.execution_info.co2_grams.toFixed(4)}g CO₂
              </span>
            </div>
          {/if}
        </div>
      </div>
    {/each}

    {#if isLoading}
      <div class="message message-assistant loading">
        <div class="message-avatar">🤖</div>
        <div class="message-content">
          <div class="loading-dots">
            <span></span><span></span><span></span>
          </div>
        </div>
      </div>
    {/if}
  </div>

  <div class="input-container">
    <textarea
      bind:value={inputMessage}
      on:keydown={handleKeydown}
      placeholder="Tapez votre message... (Entrée pour envoyer, Shift+Entrée pour nouvelle ligne)"
      rows="3"
      disabled={isLoading}
      class="message-input"
    ></textarea>
    <button
      on:click={sendMessage}
      disabled={isLoading || !inputMessage.trim()}
      class="btn-send"
    >
      {isLoading ? '⏳' : '📤'} Envoyer
    </button>
  </div>
</div>

<style>
  .mcp-chatbot {
    display: flex;
    flex-direction: column;
    height: 100vh;
    max-height: 800px;
    background: #f9fafb;
    border-radius: 12px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    overflow: hidden;
  }

  .chatbot-header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 1.5rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .chatbot-header h2 {
    margin: 0;
    font-size: 1.5rem;
  }

  .header-controls {
    display: flex;
    gap: 0.5rem;
  }

  .model-select {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    border: none;
    font-size: 0.9rem;
  }

  .btn-clear {
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.2);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background 0.2s;
  }

  .btn-clear:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .error-banner {
    background: #fee;
    color: #c00;
    padding: 1rem;
    text-align: center;
    border-bottom: 2px solid #fcc;
  }

  .quick-actions {
    display: flex;
    gap: 0.5rem;
    padding: 1rem;
    flex-wrap: wrap;
    background: white;
    border-bottom: 1px solid #e5e7eb;
  }

  .quick-action-btn {
    padding: 0.5rem 1rem;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 20px;
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.2s;
  }

  .quick-action-btn:hover:not(:disabled) {
    background: #5568d3;
    transform: translateY(-1px);
  }

  .quick-action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .empty-state {
    text-align: center;
    color: #6b7280;
    padding: 3rem 1rem;
  }

  .empty-state p {
    margin: 0.5rem 0;
  }

  .text-muted {
    font-size: 0.875rem;
    color: #9ca3af;
  }

  .message {
    display: flex;
    gap: 1rem;
    animation: slideIn 0.3s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .message-avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    flex-shrink: 0;
  }

  .message-user .message-avatar {
    background: #dbeafe;
  }

  .message-assistant .message-avatar {
    background: #f3e8ff;
  }

  .message-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .message-text {
    background: white;
    padding: 1rem;
    border-radius: 12px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .message-user .message-text {
    background: #667eea;
    color: white;
  }

  .message-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    color: #6b7280;
    padding-left: 0.5rem;
  }

  .meta-item {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .loading .message-content {
    padding: 1rem;
  }

  .loading-dots {
    display: flex;
    gap: 0.5rem;
  }

  .loading-dots span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #667eea;
    animation: bounce 1.4s infinite ease-in-out both;
  }

  .loading-dots span:nth-child(1) {
    animation-delay: -0.32s;
  }

  .loading-dots span:nth-child(2) {
    animation-delay: -0.16s;
  }

  @keyframes bounce {
    0%, 80%, 100% {
      transform: scale(0);
    }
    40% {
      transform: scale(1);
    }
  }

  .input-container {
    padding: 1rem;
    background: white;
    border-top: 1px solid #e5e7eb;
    display: flex;
    gap: 0.5rem;
  }

  .message-input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    font-family: inherit;
    font-size: 0.95rem;
    resize: none;
  }

  .message-input:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  .btn-send {
    padding: 0.75rem 1.5rem;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 600;
    transition: all 0.2s;
  }

  .btn-send:hover:not(:disabled) {
    background: #5568d3;
    transform: translateY(-1px);
  }

  .btn-send:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
